import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface WebSocketSubscription {
  channel: string
  symbol?: string
  callback: (data: any) => void
}

export interface MarketDataMessage {
  type: 'ticker' | 'kline' | 'orderbook' | 'trade'
  symbol: string
  data: any
  timestamp: number
}

export const useWebSocketStore = defineStore('websocket', () => {
  // 状态
  const socket = ref<WebSocket | null>(null)
  const isConnected = ref(false)
  const reconnectAttempts = ref(0)
  const maxReconnectAttempts = ref(5)
  const subscriptions = ref<Map<string, WebSocketSubscription>>(new Map())
  const connectionStatus = ref<'disconnected' | 'connecting' | 'connected' | 'error'>('disconnected')
  const lastError = ref<string | null>(null)
  const reconnectTimer = ref<NodeJS.Timeout | null>(null)
  
  // 实时数据存储
  const realtimeData = ref({
    symbols: new Map(),
    orderbooks: new Map(),
    klines: new Map(),
    trades: new Map()
  })

  // 计算属性
  const connectionStatusText = computed(() => {
    switch (connectionStatus.value) {
      case 'connected':
        return '已连接'
      case 'connecting':
        return '连接中...'
      case 'disconnected':
        return '未连接'
      case 'error':
        return '连接错误'
      default:
        return '未知状态'
    }
  })

  const subscriptionCount = computed(() => subscriptions.value.size)

  // 连接到Rust网关的WebSocket代理
  const connect = async (gatewayUrl: string = 'ws://localhost:8080') => {
    if (socket.value?.readyState === WebSocket.OPEN) {
      return
    }

    try {
      connectionStatus.value = 'connecting'
      
      // 连接到网关的市场数据WebSocket代理
      // 路径格式: /ws/market-data/stream
      const wsUrl = `${gatewayUrl}/ws/market-data/stream`
      
      socket.value = new WebSocket(wsUrl)

      // 连接打开事件
      socket.value.onopen = () => {
        isConnected.value = true
        connectionStatus.value = 'connected'
        reconnectAttempts.value = 0
        lastError.value = null
        console.log('WebSocket connected to Rust gateway:', wsUrl)
        
        // 发送初始订阅请求
        sendInitialSubscriptions()
      }

      // 消息接收事件
      socket.value.onmessage = (event) => {
        try {
          const message: MarketDataMessage = JSON.parse(event.data)
          handleMarketDataMessage(message)
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error)
        }
      }

      // 连接关闭事件
      socket.value.onclose = (event) => {
        isConnected.value = false
        connectionStatus.value = 'disconnected'
        console.log('WebSocket disconnected:', event.code, event.reason)
        
        // 自动重连
        if (reconnectAttempts.value < maxReconnectAttempts.value) {
          scheduleReconnect()
        }
      }

      // 连接错误事件
      socket.value.onerror = (error) => {
        connectionStatus.value = 'error'
        lastError.value = 'WebSocket连接错误'
        console.error('WebSocket error:', error)
      }

    } catch (error) {
      connectionStatus.value = 'error'
      lastError.value = error instanceof Error ? error.message : '连接失败'
      console.error('Failed to connect WebSocket:', error)
    }
  }

  // 处理市场数据消息
  const handleMarketDataMessage = (message: MarketDataMessage) => {
    const key = `${message.type}:${message.symbol}`
    const subscription = subscriptions.value.get(key)
    
    if (subscription) {
      subscription.callback(message.data)
    }

    // 更新实时数据存储
    switch (message.type) {
      case 'ticker':
        realtimeData.value.symbols.set(message.symbol, message.data)
        break
      case 'orderbook':
        realtimeData.value.orderbooks.set(message.symbol, message.data)
        break
      case 'kline':
        const klineKey = `${message.symbol}:${message.data.interval}`
        let klines = realtimeData.value.klines.get(klineKey) || []
        
        // 更新或添加K线数据
        const existingIndex = klines.findIndex((k: any) => k.openTime === message.data.openTime)
        if (existingIndex >= 0) {
          klines[existingIndex] = message.data
        } else {
          klines.push(message.data)
          // 保持最新500条记录
          if (klines.length > 500) {
            klines = klines.slice(-500)
          }
        }
        
        realtimeData.value.klines.set(klineKey, klines)
        break
      case 'trade':
        let trades = realtimeData.value.trades.get(message.symbol) || []
        trades.unshift(message.data)
        
        // 保持最新100条记录
        if (trades.length > 100) {
          trades = trades.slice(0, 100)
        }
        
        realtimeData.value.trades.set(message.symbol, trades)
        break
    }
  }

  // 发送初始订阅请求
  const sendInitialSubscriptions = () => {
    if (socket.value?.readyState !== WebSocket.OPEN) return

    try {
      // 发送订阅消息到Rust后端
      const subscribeMessage = {
        action: 'subscribe',
        channels: Array.from(subscriptions.value.keys())
      }

      socket.value.send(JSON.stringify(subscribeMessage))
    } catch (error) {
      console.error('Failed to send initial subscriptions:', error)
    }
  }

  // 计划重连
  const scheduleReconnect = () => {
    if (reconnectTimer.value) {
      clearTimeout(reconnectTimer.value)
    }

    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.value), 30000) // 指数退避，最大30秒
    reconnectAttempts.value++

    reconnectTimer.value = setTimeout(() => {
      console.log(`Attempting to reconnect... (${reconnectAttempts.value}/${maxReconnectAttempts.value})`)
      connect()
    }, delay)
  }

  // 断开连接
  const disconnect = () => {
    if (reconnectTimer.value) {
      clearTimeout(reconnectTimer.value)
      reconnectTimer.value = null
    }

    if (socket.value) {
      socket.value.close()
      socket.value = null
    }
    
    isConnected.value = false
    connectionStatus.value = 'disconnected'
    subscriptions.value.clear()
  }

  // 订阅频道
  const subscribe = (channel: string, symbol: string | null, callback: (data: any) => void) => {
    const key = symbol ? `${channel}:${symbol}` : channel
    
    subscriptions.value.set(key, {
      channel,
      symbol: symbol || undefined,
      callback
    })

    // 如果已连接，发送订阅请求
    if (socket.value?.readyState === WebSocket.OPEN) {
      const subscribeMessage = {
        action: 'subscribe',
        channel,
        symbol
      }
      socket.value.send(JSON.stringify(subscribeMessage))
    }
  }

  // 取消订阅
  const unsubscribe = (channel: string, symbol?: string) => {
    const key = symbol ? `${channel}:${symbol}` : channel
    
    subscriptions.value.delete(key)

    if (socket.value?.readyState === WebSocket.OPEN) {
      const unsubscribeMessage = {
        action: 'unsubscribe',
        channel,
        symbol
      }
      socket.value.send(JSON.stringify(unsubscribeMessage))
    }
  }

  // 取消所有订阅
  const unsubscribeAll = () => {
    subscriptions.value.clear()
    
    if (socket.value?.readyState === WebSocket.OPEN) {
      const unsubscribeMessage = {
        action: 'unsubscribe_all'
      }
      socket.value.send(JSON.stringify(unsubscribeMessage))
    }
  }

  // 发送消息到Rust后端
  const sendMessage = (message: any) => {
    if (socket.value?.readyState === WebSocket.OPEN) {
      socket.value.send(JSON.stringify(message))
    } else {
      console.warn('WebSocket not connected, cannot send message:', message)
    }
  }

  // 重连
  const reconnect = () => {
    disconnect()
    setTimeout(() => connect(), 1000)
  }

  return {
    // 状态
    socket: readonly(socket),
    isConnected: readonly(isConnected),
    connectionStatus: readonly(connectionStatus),
    reconnectAttempts: readonly(reconnectAttempts),
    lastError: readonly(lastError),
    
    // 计算属性
    connectionStatusText,
    subscriptionCount,
    
    // 方法
    connect,
    disconnect,
    subscribe,
    unsubscribe,
    unsubscribeAll,
    sendMessage,
    reconnect
  }
})