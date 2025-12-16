<template>
  <div class="binance-style-dashboard">
    <!-- 顶部价格栏 -->
    <div class="price-ticker-bar">
      <div class="main-symbol-info">
        <span class="symbol-name">{{ selectedSymbol }}</span>
        <span class="current-price" :class="priceChangeClass">{{ formatPrice(currentPrice) }}</span>
        <span class="price-change-percent" :class="priceChangeClass">{{ formatPriceChange(priceChange) }}</span>
      </div>
      
      <div class="market-stats-row">
        <div class="stat-item">
          <span class="stat-label">24h变化</span>
          <span class="stat-value" :class="priceChangeClass">{{ formatPriceChange(priceChange) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">24h高</span>
          <span class="stat-value">{{ formatPrice(high24h) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">24h低</span>
          <span class="stat-value">{{ formatPrice(low24h) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">24h量({{ selectedSymbol.replace('USDT', '') }})</span>
          <span class="stat-value">{{ formatVolume(volume24h) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">24h量(USDT)</span>
          <span class="stat-value">{{ formatVolume(quoteVolume24h) }}</span>
        </div>
      </div>
    </div>

    <!-- 主交易区域 -->
    <div class="main-trading-layout">
      <!-- 左侧：市场列表 -->
      <div class="market-sidebar">
        <div class="market-header">
          <div class="market-tabs">
            <button 
              v-for="tab in marketTabs"
              :key="tab.key"
              :class="['market-tab', { active: activeMarketTab === tab.key }]"
              @click="activeMarketTab = tab.key"
            >
              {{ tab.label }}
            </button>
          </div>
          <div class="market-search">
            <input 
              type="text" 
              placeholder="搜索"
              v-model="searchKeyword"
              class="search-input"
            />
          </div>
        </div>
        
        <div class="market-table">
          <div class="table-header">
            <div class="col-symbol">交易对</div>
            <div class="col-price">价格</div>
            <div class="col-change">涨跌幅</div>
          </div>
          <div class="table-body">
            <div 
              v-for="symbol in filteredSymbols"
              :key="symbol.symbol"
              :class="['symbol-row', { active: selectedSymbol === symbol.symbol }]"
              @click="onSymbolSelect(symbol.symbol)"
            >
              <div class="col-symbol">
                <span class="symbol-name">{{ symbol.symbol }}</span>
              </div>
              <div class="col-price">{{ formatPrice(symbol.price) }}</div>
              <div class="col-change" :class="symbol.change >= 0 ? 'positive' : 'negative'">
                {{ formatPriceChange(symbol.change) }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 中间：图表区域 -->
      <div class="chart-section">
        <div class="chart-toolbar">
          <div class="symbol-info-mini">
            <span class="symbol">{{ selectedSymbol }}</span>
            <span class="price" :class="priceChangeClass">{{ formatPrice(currentPrice) }}</span>
          </div>
          
          <div class="timeframe-selector">
            <button 
              v-for="tf in timeframes"
              :key="tf.value"
              :class="['tf-btn', { active: selectedInterval === tf.value }]"
              @click="onIntervalChange(tf.value)"
            >
              {{ tf.label }}
            </button>
          </div>
          
          <div class="chart-tools">
            <button class="tool-btn" @click="toggleFullscreen" title="全屏">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M1.5 1a.5.5 0 0 0-.5.5v4a.5.5 0 0 1-1 0v-4A1.5 1.5 0 0 1 1.5 0h4a.5.5 0 0 1 0 1h-4zM10 .5a.5.5 0 0 1 .5-.5h4A1.5 1.5 0 0 1 16 1.5v4a.5.5 0 0 1-1 0v-4a.5.5 0 0 0-.5-.5h-4a.5.5 0 0 1-.5-.5zM.5 10a.5.5 0 0 1 .5.5v4a.5.5 0 0 0 .5.5h4a.5.5 0 0 1 0 1h-4A1.5 1.5 0 0 1 0 14.5v-4a.5.5 0 0 1 .5-.5zm15 0a.5.5 0 0 1 .5.5v4a1.5 1.5 0 0 1-1.5 1.5h-4a.5.5 0 0 1 0-1h4a.5.5 0 0 0 .5-.5v-4a.5.5 0 0 1 .5-.5z"/>
              </svg>
            </button>
          </div>
        </div>
        
        <div class="chart-container">
          <TradingViewChart
            :symbol="selectedSymbol"
            :interval="selectedInterval"
            theme="dark"
            height="100%"
            @ready="onChartReady"
          />
        </div>
      </div>

      <!-- 右侧：交易和订单簿 -->
      <div class="trading-sidebar">
        <!-- 交易面板 -->
        <div class="trading-panel-section">
          <div class="panel-tabs">
            <button class="tab-btn active">现货</button>
            <button class="tab-btn">杠杆</button>
          </div>
          
          <div class="trading-form">
            <div class="order-type-tabs">
              <button class="order-type-btn active">限价</button>
              <button class="order-type-btn">市价</button>
            </div>
            
            <div class="balance-info">
              <div class="balance-row">
                <span>可用</span>
                <span class="balance-amount">{{ formatBalance(availableBalance) }} USDT</span>
              </div>
            </div>
            
            <div class="order-inputs">
              <div class="input-group">
                <label>价格</label>
                <input type="text" :value="formatPrice(currentPrice)" class="price-input" />
                <span class="input-unit">USDT</span>
              </div>
              
              <div class="input-group">
                <label>数量</label>
                <input type="text" placeholder="0" class="quantity-input" />
                <span class="input-unit">{{ selectedSymbol.replace('USDT', '') }}</span>
              </div>
              
              <div class="percentage-buttons">
                <button class="pct-btn">25%</button>
                <button class="pct-btn">50%</button>
                <button class="pct-btn">75%</button>
                <button class="pct-btn">100%</button>
              </div>
              
              <div class="input-group">
                <label>金额</label>
                <input type="text" placeholder="0" class="amount-input" />
                <span class="input-unit">USDT</span>
              </div>
            </div>
            
            <div class="order-buttons">
              <button class="buy-btn">买入 {{ selectedSymbol.replace('USDT', '') }}</button>
              <button class="sell-btn">卖出 {{ selectedSymbol.replace('USDT', '') }}</button>
            </div>
          </div>
        </div>
        
        <!-- 订单簿 -->
        <div class="orderbook-section">
          <div class="orderbook-header">
            <span>订单簿</span>
            <select class="precision-select">
              <option>0.01</option>
              <option>0.1</option>
              <option>1</option>
            </select>
          </div>
          
          <div class="orderbook-content">
            <div class="orderbook-table">
              <div class="orderbook-header-row">
                <span>价格(USDT)</span>
                <span>数量({{ selectedSymbol.replace('USDT', '') }})</span>
                <span>累计</span>
              </div>
              
              <!-- 卖单 -->
              <div class="asks-section">
                <div 
                  v-for="(ask, index) in mockAsks"
                  :key="'ask-' + index"
                  class="orderbook-row ask-row"
                  @click="onOrderBookPriceClick(ask[0])"
                >
                  <span class="price ask-price">{{ formatPrice(ask[0]) }}</span>
                  <span class="quantity">{{ formatQuantity(ask[1]) }}</span>
                  <span class="total">{{ formatQuantity(ask[2]) }}</span>
                </div>
              </div>
              
              <!-- 当前价格 -->
              <div class="current-price-row">
                <span class="current-price-label" :class="priceChangeClass">
                  {{ formatPrice(currentPrice) }}
                </span>
                <span class="price-change-mini" :class="priceChangeClass">
                  {{ formatPriceChange(priceChange) }}
                </span>
              </div>
              
              <!-- 买单 -->
              <div class="bids-section">
                <div 
                  v-for="(bid, index) in mockBids"
                  :key="'bid-' + index"
                  class="orderbook-row bid-row"
                  @click="onOrderBookPriceClick(bid[0])"
                >
                  <span class="price bid-price">{{ formatPrice(bid[0]) }}</span>
                  <span class="quantity">{{ formatQuantity(bid[1]) }}</span>
                  <span class="total">{{ formatQuantity(bid[2]) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部：订单和交易历史 -->
    <div class="bottom-panel">
      <div class="panel-tabs-bottom">
        <button 
          v-for="tab in bottomTabs"
          :key="tab.key"
          :class="['bottom-tab', { active: bottomActiveTab === tab.key }]"
          @click="bottomActiveTab = tab.key"
        >
          {{ tab.label }}
        </button>
      </div>
      
      <div class="bottom-content">
        <div v-if="bottomActiveTab === 'open-orders'" class="orders-table">
          <div class="table-header">
            <span>时间</span>
            <span>交易对</span>
            <span>类型</span>
            <span>方向</span>
            <span>数量</span>
            <span>价格</span>
            <span>成交</span>
            <span>状态</span>
            <span>操作</span>
          </div>
          <div class="table-body">
            <div class="empty-state">
              <span>暂无当前委托</span>
            </div>
          </div>
        </div>
        
        <div v-if="bottomActiveTab === 'order-history'" class="orders-table">
          <div class="table-header">
            <span>时间</span>
            <span>交易对</span>
            <span>类型</span>
            <span>方向</span>
            <span>数量</span>
            <span>价格</span>
            <span>成交</span>
            <span>状态</span>
          </div>
          <div class="table-body">
            <div class="empty-state">
              <span>暂无历史订单</span>
            </div>
          </div>
        </div>
        
        <div v-if="bottomActiveTab === 'trades'" class="trades-table">
          <div class="table-header">
            <span>时间</span>
            <span>价格</span>
            <span>数量</span>
            <span>成交额</span>
          </div>
          <div class="table-body">
            <div 
              v-for="trade in recentTrades"
              :key="trade.id"
              class="trade-row"
            >
              <span class="time">{{ formatTime(trade.time) }}</span>
              <span class="price" :class="trade.side === 'buy' ? 'positive' : 'negative'">
                {{ formatPrice(trade.price) }}
              </span>
              <span class="quantity">{{ formatQuantity(trade.quantity) }}</span>
              <span class="amount">{{ formatPrice(trade.price * trade.quantity) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import TradingViewChart from '@/components/charts/TradingViewChart.vue'
import { useTradingStore } from '@/stores/trading'
import { useMarketStore } from '@/stores/market'
import { useWebSocketStore } from '@/stores/websocket'

// 响应式数据
const selectedSymbol = ref('BTCUSDT')
const selectedInterval = ref('1m')
const activeMarketTab = ref('usdt')
const bottomActiveTab = ref('open-orders')
const searchKeyword = ref('')

// 交易相关数据 - 从真实API获取
const currentPrice = ref(0)
const priceChange = ref(0)
const priceChangePercent = ref(0)
const high24h = ref(0)
const low24h = ref(0)
const volume24h = ref(0)
const quoteVolume24h = ref(0)
const availableBalance = ref(10000)

// 配置数据
const marketTabs = [
  { key: 'usdt', label: 'USDT' },
  { key: 'btc', label: 'BTC' },
  { key: 'eth', label: 'ETH' },
  { key: 'hot', label: '热门' }
]

const bottomTabs = [
  { key: 'open-orders', label: '当前委托' },
  { key: 'order-history', label: '订单历史' },
  { key: 'trades', label: '成交记录' }
]

const timeframes = [
  { label: '1m', value: '1m' },
  { label: '5m', value: '5m' },
  { label: '15m', value: '15m' },
  { label: '30m', value: '30m' },
  { label: '1H', value: '1h' },
  { label: '4H', value: '4h' },
  { label: '1D', value: '1d' }
]

// 真实交易对数据
const allSymbols = ref<any[]>([])

const mockBids = ref([
  [49950, 0.5, 0.5],
  [49940, 1.2, 1.7],
  [49930, 0.8, 2.5],
  [49920, 2.1, 4.6],
  [49910, 1.5, 6.1]
])

const mockAsks = ref([
  [50050, 0.3, 0.3],
  [50060, 0.9, 1.2],
  [50070, 1.1, 2.3],
  [50080, 0.7, 3.0],
  [50090, 2.0, 5.0]
])

const recentTrades = ref([
  { id: 1, time: Date.now(), price: 50000, quantity: 0.1, side: 'buy' },
  { id: 2, time: Date.now() - 1000, price: 49950, quantity: 0.2, side: 'sell' },
  { id: 3, time: Date.now() - 2000, price: 50100, quantity: 0.05, side: 'buy' }
])

// 状态管理
const tradingStore = useTradingStore()
const marketStore = useMarketStore()
const wsStore = useWebSocketStore()

// 计算属性
const filteredSymbols = computed(() => {
  let symbols = allSymbols.value
  
  if (activeMarketTab.value === 'usdt') {
    symbols = symbols.filter(s => s.symbol.endsWith('USDT'))
  } else if (activeMarketTab.value === 'btc') {
    symbols = symbols.filter(s => s.symbol.endsWith('BTC'))
  } else if (activeMarketTab.value === 'eth') {
    symbols = symbols.filter(s => s.symbol.endsWith('ETH'))
  }
  
  if (searchKeyword.value) {
    symbols = symbols.filter(s => 
      s.symbol.toLowerCase().includes(searchKeyword.value.toLowerCase())
    )
  }
  
  return symbols
})

const priceChangeClass = computed(() => {
  return priceChange.value >= 0 ? 'positive' : 'negative'
})

// 事件处理
const onSymbolSelect = (symbol: string) => {
  selectedSymbol.value = symbol
  const symbolData = allSymbols.value.find(s => s.symbol === symbol)
  if (symbolData) {
    currentPrice.value = symbolData.price
    priceChange.value = symbolData.change
  }
}

const onIntervalChange = (interval: string) => {
  selectedInterval.value = interval
}

const onChartReady = (chart: any) => {
  console.log('Chart ready:', chart)
}

const onOrderBookPriceClick = (price: number) => {
  console.log('Order book price clicked:', price)
}

const toggleFullscreen = () => {
  if (document.fullscreenElement) {
    document.exitFullscreen()
  } else {
    document.documentElement.requestFullscreen()
  }
}

// 工具函数
const formatPrice = (price: number) => {
  return price.toLocaleString('en-US', { 
    minimumFractionDigits: 2, 
    maximumFractionDigits: 2 
  })
}

const formatPriceChange = (change: number) => {
  const sign = change >= 0 ? '+' : ''
  return `${sign}${change.toFixed(2)}%`
}

const formatVolume = (volume: number) => {
  if (volume >= 1e9) {
    return (volume / 1e9).toFixed(2) + 'B'
  } else if (volume >= 1e6) {
    return (volume / 1e6).toFixed(2) + 'M'
  } else if (volume >= 1e3) {
    return (volume / 1e3).toFixed(2) + 'K'
  }
  return volume.toFixed(2)
}

const formatQuantity = (quantity: number) => {
  return quantity.toFixed(4)
}

const formatBalance = (balance: number) => {
  return balance.toFixed(2)
}

const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString()
}

// 实时数据更新
let priceUpdateTimer: number | null = null
let orderbookUpdateTimer: number | null = null

// 启动实时数据更新
const startRealTimeUpdates = () => {
  // 连接WebSocket
  if (wsStore.connect) {
    wsStore.connect().then(() => {
      console.log('WebSocket connected successfully')
      
      // 订阅实时数据
      if (wsStore.subscribe) {
        wsStore.subscribe('ticker', selectedSymbol.value, (data: any) => {
          if (Array.isArray(data)) {
            // 更新所有交易对数据
            allSymbols.value = data.map((item: any) => ({
              symbol: item.symbol,
              price: item.price,
              change: item.change
            }))
            
            // 更新当前选中交易对的价格
            const currentSymbolData = data.find((item: any) => item.symbol === selectedSymbol.value)
            if (currentSymbolData) {
              currentPrice.value = currentSymbolData.price
              priceChange.value = currentSymbolData.change
              high24h.value = currentSymbolData.high
              low24h.value = currentSymbolData.low
              volume24h.value = currentSymbolData.volume
            }
          }
        })
        
        wsStore.subscribe('orderbook', selectedSymbol.value, (data: any) => {
          if (data.bids && data.asks) {
            mockBids.value = data.bids.slice(0, 10).map((bid: any, index: number) => [
              parseFloat(bid[0]),
              parseFloat(bid[1]),
              parseFloat(bid[1]) * (index + 1) // 累计量
            ])
            
            mockAsks.value = data.asks.slice(0, 10).map((ask: any, index: number) => [
              parseFloat(ask[0]),
              parseFloat(ask[1]),
              parseFloat(ask[1]) * (index + 1) // 累计量
            ])
          }
        })
        
        wsStore.subscribe('trades', selectedSymbol.value, (data: any) => {
          if (data.trades) {
            recentTrades.value = data.trades.map((trade: any) => ({
              id: trade.id,
              time: trade.time,
              price: trade.price,
              quantity: trade.quantity,
              side: trade.isBuyerMaker ? 'sell' : 'buy'
            }))
          }
        })
      }
    }).catch((error) => {
      console.warn('WebSocket connection failed:', error)
    })
  }
  
  // 模拟价格波动 (作为备用)
  priceUpdateTimer = setInterval(() => {
    // 模拟价格小幅波动
    const change = (Math.random() - 0.5) * 0.002 // ±0.1%
    currentPrice.value = Math.max(currentPrice.value * (1 + change), 1)
    
    // 更新交易对列表中的价格
    allSymbols.value.forEach(symbol => {
      const symbolChange = (Math.random() - 0.5) * 0.003
      symbol.price = Math.max(symbol.price * (1 + symbolChange), 0.01)
      symbol.change = symbol.change + (Math.random() - 0.5) * 0.1
    })
  }, 3000)
  
  // 模拟订单簿更新
  orderbookUpdateTimer = setInterval(() => {
    // 更新买单
    mockBids.value.forEach(bid => {
      bid[0] = bid[0] * (1 + (Math.random() - 0.5) * 0.0005)
      bid[1] = Math.max(bid[1] * (1 + (Math.random() - 0.5) * 0.1), 0.01)
    })
    
    // 更新卖单
    mockAsks.value.forEach(ask => {
      ask[0] = ask[0] * (1 + (Math.random() - 0.5) * 0.0005)
      ask[1] = Math.max(ask[1] * (1 + (Math.random() - 0.5) * 0.1), 0.01)
    })
  }, 2000)
}

// 停止实时数据更新
const stopRealTimeUpdates = () => {
  if (priceUpdateTimer) {
    clearInterval(priceUpdateTimer)
    priceUpdateTimer = null
  }
  
  if (orderbookUpdateTimer) {
    clearInterval(orderbookUpdateTimer)
    orderbookUpdateTimer = null
  }
  
  if (wsStore.disconnect) {
    wsStore.disconnect()
  }
}

// 获取真实市场数据
const loadRealMarketData = async () => {
  try {
    // 获取真实的价格数据
    const response = await fetch('http://localhost:8081/api/v1/tickers')
    const result = await response.json()
    
    if (result.success && result.data) {
      allSymbols.value = result.data.map((item: any) => ({
        symbol: item.symbol,
        price: parseFloat(item.price),
        change: parseFloat(item.change),
        volume: parseFloat(item.volume),
        quoteVolume: parseFloat(item.quoteVolume),
        high: parseFloat(item.high),
        low: parseFloat(item.low),
        open: parseFloat(item.open)
      }))
      
      // 更新当前选中交易对的数据
      const currentSymbolData = allSymbols.value.find(s => s.symbol === selectedSymbol.value)
      if (currentSymbolData) {
        // 确保所有价格显示都同步更新
        currentPrice.value = currentSymbolData.price
        priceChange.value = currentSymbolData.change
        priceChangePercent.value = currentSymbolData.change
        high24h.value = currentSymbolData.high
        low24h.value = currentSymbolData.low
        volume24h.value = currentSymbolData.volume
        quoteVolume24h.value = currentSymbolData.quoteVolume
        
        // 更新订单簿的当前价格显示
        updateOrderBookPrices(currentSymbolData.price)
        
        console.log(`✅ Updated ${selectedSymbol.value} price: $${currentPrice.value.toLocaleString()}`)
      }
      
      console.log('✅ Loaded real market data:', allSymbols.value.length, 'symbols')
      console.log('📊 BTC Price:', allSymbols.value.find(s => s.symbol === 'BTCUSDT')?.price)
    }
  } catch (error) {
    console.error('❌ Failed to load real market data:', error)
  }
}

// 更新订单簿价格（基于真实价格生成合理的买卖盘）
const updateOrderBookPrices = (basePrice: number) => {
  // 生成基于真实价格的买单（略低于当前价格）
  mockBids.value = Array.from({ length: 10 }, (_, i) => {
    const priceOffset = (i + 1) * 0.001 // 0.1% 递减
    const price = basePrice * (1 - priceOffset)
    const quantity = Math.random() * 2 + 0.1
    return [price, quantity, quantity * (i + 1)]
  })
  
  // 生成基于真实价格的卖单（略高于当前价格）
  mockAsks.value = Array.from({ length: 10 }, (_, i) => {
    const priceOffset = (i + 1) * 0.001 // 0.1% 递增
    const price = basePrice * (1 + priceOffset)
    const quantity = Math.random() * 2 + 0.1
    return [price, quantity, quantity * (i + 1)]
  })
}

// 生命周期
onMounted(async () => {
  console.log('Trading dashboard mounted')
  
  // 首先加载真实市场数据
  await loadRealMarketData()
  
  try {
    // 连接到Rust网关的WebSocket代理
    await wsStore.connect('ws://localhost:8080')
    
    // 订阅市场数据
    wsStore.subscribe('ticker', selectedSymbol.value, (data) => {
      // 更新价格数据
      currentPrice.value = data.price || currentPrice.value
      priceChange.value = data.priceChange || priceChange.value
      priceChangePercent.value = data.priceChangePercent || priceChangePercent.value
      volume24h.value = data.volume || volume24h.value
    })
    
    wsStore.subscribe('orderbook', selectedSymbol.value, (data) => {
      // 更新订单簿数据
      if (data.bids) mockBids.value = data.bids.slice(0, 10)
      if (data.asks) mockAsks.value = data.asks.slice(0, 10)
    })
    
    wsStore.subscribe('kline', selectedSymbol.value, (data) => {
      // 更新K线图表
      if (chartComponent.value) {
        chartComponent.value.updateKline(data)
      }
    })
    
    wsStore.subscribe('trade', selectedSymbol.value, (data) => {
      // 更新最新成交
      recentTrades.value.unshift(data)
      if (recentTrades.value.length > 50) {
        recentTrades.value = recentTrades.value.slice(0, 50)
      }
    })
    
    console.log('Connected to Rust microservices with real data')
  } catch (error) {
    console.error('Failed to connect to Rust services:', error)
  }
  
  // 实时刷新真实数据 - 毫秒级更新
  setInterval(loadRealMarketData, 1000) // 每1秒刷新一次，模拟实时更新
})

onUnmounted(() => {
  console.log('Trading dashboard unmounted')
  stopRealTimeUpdates()
})
</script>

<style lang="scss" scoped>
.binance-style-dashboard {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #0b0e11;
  color: #eaecef;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 12px;
}

// 顶部价格栏
.price-ticker-bar {
  height: 60px;
  background: #1e2329;
  border-bottom: 1px solid #2b3139;
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 32px;
  
  .main-symbol-info {
    display: flex;
    align-items: center;
    gap: 12px;
    
    .symbol-name {
      font-size: 16px;
      font-weight: 600;
      color: #eaecef;
    }
    
    .current-price {
      font-size: 20px;
      font-weight: 600;
      font-family: 'SF Mono', Monaco, monospace;
      
      &.positive { color: #02c076; }
      &.negative { color: #f84960; }
    }
    
    .price-change-percent {
      font-size: 14px;
      font-weight: 500;
      padding: 2px 6px;
      border-radius: 2px;
      
      &.positive { 
        color: #02c076; 
        background: rgba(2, 192, 118, 0.1);
      }
      &.negative { 
        color: #f84960; 
        background: rgba(248, 73, 96, 0.1);
      }
    }
  }
  
  .market-stats-row {
    display: flex;
    gap: 24px;
    
    .stat-item {
      display: flex;
      flex-direction: column;
      gap: 2px;
      
      .stat-label {
        font-size: 11px;
        color: #848e9c;
      }
      
      .stat-value {
        font-size: 12px;
        font-weight: 500;
        color: #eaecef;
        font-family: 'SF Mono', Monaco, monospace;
        
        &.positive { color: #02c076; }
        &.negative { color: #f84960; }
      }
    }
  }
}

// 主交易布局
.main-trading-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}

// 左侧市场列表
.market-sidebar {
  width: 280px;
  background: #1e2329;
  border-right: 1px solid #2b3139;
  display: flex;
  flex-direction: column;
  
  .market-header {
    padding: 12px;
    border-bottom: 1px solid #2b3139;
    
    .market-tabs {
      display: flex;
      gap: 1px;
      margin-bottom: 12px;
      
      .market-tab {
        flex: 1;
        padding: 6px 12px;
        background: #2b3139;
        border: none;
        color: #848e9c;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
        
        &:first-child { border-radius: 2px 0 0 2px; }
        &:last-child { border-radius: 0 2px 2px 0; }
        
        &.active {
          background: #f0b90b;
          color: #000;
        }
        
        &:hover:not(.active) {
          background: #3c4043;
          color: #eaecef;
        }
      }
    }
    
    .market-search {
      .search-input {
        width: 100%;
        padding: 6px 8px;
        background: #2b3139;
        border: 1px solid #3c4043;
        border-radius: 2px;
        color: #eaecef;
        font-size: 12px;
        
        &::placeholder {
          color: #848e9c;
        }
        
        &:focus {
          outline: none;
          border-color: #f0b90b;
        }
      }
    }
  }
  
  .market-table {
    flex: 1;
    overflow: hidden;
    
    .table-header {
      display: grid;
      grid-template-columns: 1fr 80px 60px;
      padding: 8px 12px;
      background: #2b3139;
      font-size: 11px;
      color: #848e9c;
      font-weight: 500;
      
      .col-symbol { text-align: left; }
      .col-price { text-align: right; }
      .col-change { text-align: right; }
    }
    
    .table-body {
      height: calc(100% - 32px);
      overflow-y: auto;
      
      .symbol-row {
        display: grid;
        grid-template-columns: 1fr 80px 60px;
        padding: 6px 12px;
        cursor: pointer;
        transition: background 0.2s;
        
        &:hover {
          background: #2b3139;
        }
        
        &.active {
          background: rgba(240, 185, 11, 0.1);
        }
        
        .col-symbol {
          .symbol-name {
            font-size: 12px;
            font-weight: 500;
            color: #eaecef;
          }
        }
        
        .col-price {
          text-align: right;
          font-family: 'SF Mono', Monaco, monospace;
          font-size: 12px;
          color: #eaecef;
        }
        
        .col-change {
          text-align: right;
          font-family: 'SF Mono', Monaco, monospace;
          font-size: 12px;
          
          &.positive { color: #02c076; }
          &.negative { color: #f84960; }
        }
      }
    }
  }
}

// 中间图表区域
.chart-section {
  flex: 1;
  background: #1e2329;
  display: flex;
  flex-direction: column;
  min-width: 0;
  
  .chart-toolbar {
    height: 40px;
    background: #2b3139;
    border-bottom: 1px solid #3c4043;
    display: flex;
    align-items: center;
    padding: 0 12px;
    gap: 16px;
    
    .symbol-info-mini {
      display: flex;
      align-items: center;
      gap: 8px;
      
      .symbol {
        font-size: 14px;
        font-weight: 600;
        color: #eaecef;
      }
      
      .price {
        font-size: 14px;
        font-weight: 600;
        font-family: 'SF Mono', Monaco, monospace;
        
        &.positive { color: #02c076; }
        &.negative { color: #f84960; }
      }
    }
    
    .timeframe-selector {
      display: flex;
      gap: 1px;
      
      .tf-btn {
        padding: 4px 8px;
        background: transparent;
        border: 1px solid #3c4043;
        color: #848e9c;
        font-size: 11px;
        cursor: pointer;
        transition: all 0.2s;
        
        &:first-child { border-radius: 2px 0 0 2px; }
        &:last-child { border-radius: 0 2px 2px 0; }
        
        &.active {
          background: #f0b90b;
          border-color: #f0b90b;
          color: #000;
        }
        
        &:hover:not(.active) {
          background: #3c4043;
          color: #eaecef;
        }
      }
    }
    
    .chart-tools {
      margin-left: auto;
      display: flex;
      align-items: center;
      gap: 12px;
      
      .realtime-indicator {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 4px 8px;
        background: rgba(43, 49, 57, 0.5);
        border-radius: 12px;
        
        .indicator-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          
          &.connected {
            background: #02c076;
            box-shadow: 0 0 6px rgba(2, 192, 118, 0.6);
            animation: pulse 2s infinite;
          }
          
          &.connecting {
            background: #f0b90b;
            animation: pulse 1s infinite;
          }
          
          &.disconnected {
            background: #848e9c;
          }
        }
        
        .indicator-text {
          font-size: 11px;
          color: #848e9c;
          font-weight: 500;
        }
      }
      
      .tool-btn {
        padding: 6px;
        background: transparent;
        border: 1px solid #3c4043;
        border-radius: 2px;
        color: #848e9c;
        cursor: pointer;
        transition: all 0.2s;
        
        &:hover {
          background: #3c4043;
          color: #eaecef;
        }
      }
    }
  }
  
  .chart-container {
    flex: 1;
    background: #1e2329;
  }
}

// 右侧交易面板
.trading-sidebar {
  width: 320px;
  background: #1e2329;
  border-left: 1px solid #2b3139;
  display: flex;
  flex-direction: column;
  
  .trading-panel-section {
    border-bottom: 1px solid #2b3139;
    
    .panel-tabs {
      display: flex;
      background: #2b3139;
      
      .tab-btn {
        flex: 1;
        padding: 8px 16px;
        background: transparent;
        border: none;
        color: #848e9c;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
        
        &.active {
          background: #1e2329;
          color: #eaecef;
        }
      }
    }
    
    .trading-form {
      padding: 16px;
      
      .order-type-tabs {
        display: flex;
        gap: 1px;
        margin-bottom: 16px;
        
        .order-type-btn {
          flex: 1;
          padding: 6px 12px;
          background: #2b3139;
          border: none;
          color: #848e9c;
          font-size: 12px;
          cursor: pointer;
          
          &:first-child { border-radius: 2px 0 0 2px; }
          &:last-child { border-radius: 0 2px 2px 0; }
          
          &.active {
            background: #f0b90b;
            color: #000;
          }
        }
      }
      
      .balance-info {
        margin-bottom: 16px;
        
        .balance-row {
          display: flex;
          justify-content: space-between;
          font-size: 12px;
          color: #848e9c;
          
          .balance-amount {
            color: #eaecef;
            font-family: 'SF Mono', Monaco, monospace;
          }
        }
      }
      
      .order-inputs {
        .input-group {
          margin-bottom: 12px;
          
          label {
            display: block;
            font-size: 11px;
            color: #848e9c;
            margin-bottom: 4px;
          }
          
          position: relative;
          
          input {
            width: 100%;
            padding: 8px 40px 8px 8px;
            background: #2b3139;
            border: 1px solid #3c4043;
            border-radius: 2px;
            color: #eaecef;
            font-size: 12px;
            font-family: 'SF Mono', Monaco, monospace;
            
            &:focus {
              outline: none;
              border-color: #f0b90b;
            }
          }
          
          .input-unit {
            position: absolute;
            right: 8px;
            top: 50%;
            transform: translateY(-50%);
            font-size: 11px;
            color: #848e9c;
            pointer-events: none;
          }
        }
        
        .percentage-buttons {
          display: flex;
          gap: 4px;
          margin-bottom: 12px;
          
          .pct-btn {
            flex: 1;
            padding: 4px 8px;
            background: #2b3139;
            border: 1px solid #3c4043;
            border-radius: 2px;
            color: #848e9c;
            font-size: 11px;
            cursor: pointer;
            transition: all 0.2s;
            
            &:hover {
              background: #3c4043;
              color: #eaecef;
            }
          }
        }
      }
      
      .order-buttons {
        display: flex;
        gap: 8px;
        
        .buy-btn, .sell-btn {
          flex: 1;
          padding: 10px 16px;
          border: none;
          border-radius: 2px;
          font-size: 12px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }
        
        .buy-btn {
          background: #02c076;
          color: #fff;
          
          &:hover {
            background: #00a866;
          }
        }
        
        .sell-btn {
          background: #f84960;
          color: #fff;
          
          &:hover {
            background: #e63946;
          }
        }
      }
    }
  }
  
  .orderbook-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    
    .orderbook-header {
      padding: 12px 16px;
      background: #2b3139;
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 12px;
      font-weight: 500;
      color: #eaecef;
      
      .precision-select {
        background: #3c4043;
        border: 1px solid #4a5568;
        border-radius: 2px;
        color: #eaecef;
        font-size: 11px;
        padding: 2px 6px;
      }
    }
    
    .orderbook-content {
      flex: 1;
      overflow: hidden;
      
      .orderbook-table {
        height: 100%;
        display: flex;
        flex-direction: column;
        
        .orderbook-header-row {
          display: grid;
          grid-template-columns: 1fr 1fr 1fr;
          padding: 8px 16px;
          background: #2b3139;
          font-size: 11px;
          color: #848e9c;
          text-align: right;
          
          span:first-child {
            text-align: left;
          }
        }
        
        .asks-section, .bids-section {
          .orderbook-row {
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            padding: 2px 16px;
            font-size: 11px;
            font-family: 'SF Mono', Monaco, monospace;
            cursor: pointer;
            transition: background 0.2s;
            
            &:hover {
              background: #2b3139;
            }
            
            .price {
              text-align: left;
              
              &.ask-price { color: #f84960; }
              &.bid-price { color: #02c076; }
            }
            
            .quantity, .total {
              text-align: right;
              color: #eaecef;
            }
          }
        }
        
        .current-price-row {
          padding: 8px 16px;
          background: #2b3139;
          display: flex;
          justify-content: space-between;
          align-items: center;
          border-top: 1px solid #3c4043;
          border-bottom: 1px solid #3c4043;
          
          .current-price-label {
            font-size: 14px;
            font-weight: 600;
            font-family: 'SF Mono', Monaco, monospace;
            
            &.positive { color: #02c076; }
            &.negative { color: #f84960; }
          }
          
          .price-change-mini {
            font-size: 11px;
            
            &.positive { color: #02c076; }
            &.negative { color: #f84960; }
          }
        }
      }
    }
  }
}

// 底部面板
.bottom-panel {
  height: 200px;
  background: #1e2329;
  border-top: 1px solid #2b3139;
  display: flex;
  flex-direction: column;
  
  .panel-tabs-bottom {
    display: flex;
    background: #2b3139;
    
    .bottom-tab {
      padding: 8px 16px;
      background: transparent;
      border: none;
      color: #848e9c;
      font-size: 12px;
      cursor: pointer;
      transition: all 0.2s;
      
      &.active {
        background: #1e2329;
        color: #eaecef;
        border-bottom: 2px solid #f0b90b;
      }
    }
  }
  
  .bottom-content {
    flex: 1;
    overflow: hidden;
    
    .orders-table, .trades-table {
      height: 100%;
      display: flex;
      flex-direction: column;
      
      .table-header {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
        padding: 8px 16px;
        background: #2b3139;
        font-size: 11px;
        color: #848e9c;
        font-weight: 500;
      }
      
      .table-body {
        flex: 1;
        overflow-y: auto;
        
        .empty-state {
          display: flex;
          justify-content: center;
          align-items: center;
          height: 100px;
          color: #848e9c;
          font-size: 12px;
        }
        
        .trade-row {
          display: grid;
          grid-template-columns: 80px 100px 100px 100px;
          padding: 4px 16px;
          font-size: 11px;
          font-family: 'SF Mono', Monaco, monospace;
          
          .time {
            color: #848e9c;
          }
          
          .price {
            &.positive { color: #02c076; }
            &.negative { color: #f84960; }
          }
          
          .quantity, .amount {
            color: #eaecef;
          }
        }
      }
    }
  }
}

// 滚动条样式
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: #2b3139;
}

::-webkit-scrollbar-thumb {
  background: #3c4043;
  border-radius: 3px;
  
  &:hover {
    background: #4a5568;
  }
}

// 响应式
@media (max-width: 1200px) {
  .market-sidebar {
    width: 240px;
  }
  
  .trading-sidebar {
    width: 280px;
  }
}
</style>