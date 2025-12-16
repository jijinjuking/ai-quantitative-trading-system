import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { marketDataApi } from '@/utils/api'

export interface Symbol {
  symbol: string
  baseAsset: string
  quoteAsset: string
  status: string
  minPrice: number
  maxPrice: number
  tickSize: number
  minQty: number
  maxQty: number
  stepSize: number
}

export interface Ticker {
  symbol: string
  price: number
  lastPrice: number
  priceChange: number
  priceChangePercent: number
  volume: number
  quoteVolume: number
  high: number
  low: number
  open: number
  close: number
  count: number
  timestamp: number
}

export interface OrderBookLevel {
  price: number
  quantity: number
  total?: number
}

export interface OrderBook {
  symbol: string
  bids: OrderBookLevel[]
  asks: OrderBookLevel[]
  lastUpdateId: number
  timestamp: number
}

export interface Trade {
  id: number
  symbol: string
  price: number
  quantity: number
  time: number
  isBuyerMaker: boolean
  side: 'buy' | 'sell'
}

export interface KlineData {
  symbol: string
  interval: string
  openTime: number
  closeTime: number
  open: number
  high: number
  low: number
  close: number
  volume: number
  quoteVolume: number
  trades: number
}

export const useMarketStore = defineStore('market', () => {
  // 状态
  const symbols = ref<Symbol[]>([])
  const tickers = ref<Map<string, Ticker>>(new Map())
  const orderBooks = ref<Map<string, OrderBook>>(new Map())
  const recentTrades = ref<Map<string, Trade[]>>(new Map())
  const klineData = ref<Map<string, KlineData[]>>(new Map())
  const isLoading = ref(false)
  const lastError = ref<string | null>(null)
  
  // 市场统计
  const marketStats = ref({
    totalVolume24h: 0,
    totalMarkets: 0,
    gainers: 0,
    losers: 0,
    unchanged: 0
  })

  // 计算属性
  const marketSymbols = computed(() => {
    return symbols.value.filter(s => s.status === 'TRADING')
  })

  const topGainers = computed(() => {
    return Array.from(tickers.value.values())
      .filter(t => t.priceChangePercent > 0)
      .sort((a, b) => b.priceChangePercent - a.priceChangePercent)
      .slice(0, 10)
  })

  const topLosers = computed(() => {
    return Array.from(tickers.value.values())
      .filter(t => t.priceChangePercent < 0)
      .sort((a, b) => a.priceChangePercent - b.priceChangePercent)
      .slice(0, 10)
  })

  const topVolume = computed(() => {
    return Array.from(tickers.value.values())
      .sort((a, b) => b.quoteVolume - a.quoteVolume)
      .slice(0, 10)
  })

  // 获取交易对信息 - 通过网关代理到市场数据服务
  const loadMarketData = async () => {
    try {
      isLoading.value = true
      lastError.value = null

      // 通过网关获取交易对信息
      // 网关会将请求代理到市场数据服务 (localhost:8083)
      const symbolsResponse = await marketDataApi.getSymbols()
      symbols.value = symbolsResponse.data

      // 获取24小时价格统计
      const tickersResponse = await marketDataApi.getTickers()
      const tickerData = tickersResponse.data
      
      tickers.value.clear()
      tickerData.forEach((ticker: Ticker) => {
        tickers.value.set(ticker.symbol, ticker)
      })

      // 更新市场统计
      updateMarketStats()

    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '加载市场数据失败'
      console.error('Failed to load market data:', error)
    } finally {
      isLoading.value = false
    }
  }

  // 获取订单簿 - 通过网关代理
  const getOrderBook = async (symbol: string, limit: number = 20): Promise<OrderBook> => {
    try {
      const response = await marketDataApi.getOrderBook(symbol, limit)
      
      const orderBook = response.data
      orderBooks.value.set(symbol, orderBook)
      
      return orderBook
    } catch (error) {
      console.error('Failed to get orderbook:', error)
      throw error
    }
  }

  // 获取最新成交 - 通过网关代理
  const getRecentTrades = async (symbol: string, limit: number = 50): Promise<Trade[]> => {
    try {
      const response = await marketDataApi.getTrades(symbol, limit)
      
      const trades = response.data
      recentTrades.value.set(symbol, trades)
      
      return trades
    } catch (error) {
      console.error('Failed to get recent trades:', error)
      throw error
    }
  }

  // 获取K线数据 - 通过网关代理
  const getKlineData = async (
    symbol: string, 
    interval: string, 
    limit: number = 500
  ): Promise<KlineData[]> => {
    try {
      const response = await marketDataApi.getKlines(symbol, interval, limit)
      
      const klines = response.data
      const key = `${symbol}:${interval}`
      klineData.value.set(key, klines)
      
      return klines
    } catch (error) {
      console.error('Failed to get kline data:', error)
      throw error
    }
  }

  // 获取单个价格信息 - 通过网关代理
  const getTicker = async (symbol: string): Promise<Ticker> => {
    try {
      const response = await marketDataApi.getTicker(symbol)
      const ticker = response.data
      
      tickers.value.set(symbol, ticker)
      return ticker
    } catch (error) {
      console.error('Failed to get ticker:', error)
      throw error
    }
  }

  // 搜索交易对
  const searchSymbols = (query: string): Symbol[] => {
    if (!query) return marketSymbols.value
    
    const lowerQuery = query.toLowerCase()
    return marketSymbols.value.filter(symbol => 
      symbol.symbol.toLowerCase().includes(lowerQuery) ||
      symbol.baseAsset.toLowerCase().includes(lowerQuery) ||
      symbol.quoteAsset.toLowerCase().includes(lowerQuery)
    )
  }

  // 按分类获取交易对
  const getSymbolsByCategory = (category: string): Symbol[] => {
    switch (category) {
      case 'USDT':
        return marketSymbols.value.filter(s => s.quoteAsset === 'USDT')
      case 'BTC':
        return marketSymbols.value.filter(s => s.quoteAsset === 'BTC')
      case 'ETH':
        return marketSymbols.value.filter(s => s.quoteAsset === 'ETH')
      case 'HOT':
        // 热门交易对（按成交量排序）
        const hotSymbols = Array.from(tickers.value.values())
          .sort((a, b) => b.quoteVolume - a.quoteVolume)
          .slice(0, 20)
          .map(t => t.symbol)
        return marketSymbols.value.filter(s => hotSymbols.includes(s.symbol))
      default:
        return marketSymbols.value
    }
  }

  // 更新订单簿
  const updateOrderbook = (symbol: string, data: any) => {
    const currentOrderbook = orderBooks.value.get(symbol)
    if (currentOrderbook) {
      // 应用增量更新
      const updatedOrderbook = {
        ...currentOrderbook,
        bids: data.bids || currentOrderbook.bids,
        asks: data.asks || currentOrderbook.asks,
        lastUpdateId: data.lastUpdateId || currentOrderbook.lastUpdateId,
        timestamp: Date.now()
      }
      orderBooks.value.set(symbol, updatedOrderbook)
    }
  }

  // 更新价格信息
  const updateTicker = (symbol: string, data: Partial<Ticker>) => {
    const currentTicker = tickers.value.get(symbol)
    if (currentTicker) {
      const updatedTicker = { ...currentTicker, ...data }
      tickers.value.set(symbol, updatedTicker)
    }
  }

  // 添加新成交记录
  const addTrade = (symbol: string, trade: Trade) => {
    const trades = recentTrades.value.get(symbol) || []
    trades.unshift(trade)
    
    // 保持最新50条记录
    if (trades.length > 50) {
      trades.splice(50)
    }
    
    recentTrades.value.set(symbol, trades)
  }

  // 更新市场统计
  const updateMarketStats = () => {
    const tickerArray = Array.from(tickers.value.values())
    
    marketStats.value = {
      totalVolume24h: tickerArray.reduce((sum, t) => sum + t.quoteVolume, 0),
      totalMarkets: tickerArray.length,
      gainers: tickerArray.filter(t => t.priceChangePercent > 0).length,
      losers: tickerArray.filter(t => t.priceChangePercent < 0).length,
      unchanged: tickerArray.filter(t => t.priceChangePercent === 0).length
    }
  }

  // 刷新市场数据
  const refreshMarketData = async () => {
    await loadMarketData()
  }

  // 实时更新数据
  const updateRealTimeData = () => {
    // 这里可以添加定时更新逻辑
    updateMarketStats()
  }

  // 获取交易对详情
  const getSymbolInfo = (symbol: string): Symbol | undefined => {
    return symbols.value.find(s => s.symbol === symbol)
  }

  // 获取价格精度
  const getPricePrecision = (symbol: string): number => {
    const symbolInfo = getSymbolInfo(symbol)
    if (!symbolInfo) return 2
    
    const tickSize = symbolInfo.tickSize
    return tickSize.toString().split('.')[1]?.length || 0
  }

  // 获取数量精度
  const getQuantityPrecision = (symbol: string): number => {
    const symbolInfo = getSymbolInfo(symbol)
    if (!symbolInfo) return 4
    
    const stepSize = symbolInfo.stepSize
    return stepSize.toString().split('.')[1]?.length || 0
  }

  return {
    // 状态
    symbols: readonly(symbols),
    tickers: readonly(tickers),
    orderBooks: readonly(orderBooks),
    recentTrades: readonly(recentTrades),
    klineData: readonly(klineData),
    isLoading: readonly(isLoading),
    lastError: readonly(lastError),
    marketStats: readonly(marketStats),
    
    // 计算属性
    marketSymbols,
    topGainers,
    topLosers,
    topVolume,
    
    // 方法
    loadMarketData,
    getOrderBook,
    getRecentTrades,
    getKlineData,
    getTicker,
    searchSymbols,
    getSymbolsByCategory,
    updateOrderbook,
    updateTicker,
    addTrade,
    refreshMarketData,
    updateRealTimeData,
    getSymbolInfo,
    getPricePrecision,
    getQuantityPrecision
  }
})