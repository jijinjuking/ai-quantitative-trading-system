import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

export interface AIAnalysis {
  symbol: string
  timestamp: number
  technicalAnalysis: {
    trend: 'BULLISH' | 'BEARISH' | 'NEUTRAL'
    strength: number // 0-100
    support: number[]
    resistance: number[]
    indicators: {
      rsi: number
      macd: { value: number; signal: number; histogram: number }
      bollinger: { upper: number; middle: number; lower: number }
      ma: { ma20: number; ma50: number; ma200: number }
    }
  }
  fundamentalAnalysis: {
    sentiment: 'POSITIVE' | 'NEGATIVE' | 'NEUTRAL'
    score: number // 0-100
    factors: string[]
    news: Array<{
      title: string
      sentiment: 'POSITIVE' | 'NEGATIVE' | 'NEUTRAL'
      impact: 'HIGH' | 'MEDIUM' | 'LOW'
      timestamp: number
    }>
  }
  prediction: {
    direction: 'UP' | 'DOWN' | 'SIDEWAYS'
    confidence: number // 0-100
    timeframe: '1h' | '4h' | '1d' | '1w'
    targetPrice: number
    stopLoss: number
    reasoning: string
  }
  signals: Array<{
    type: 'BUY' | 'SELL' | 'HOLD'
    strength: number // 0-100
    source: string
    description: string
    timestamp: number
  }>
}

export interface AIStrategy {
  id: string
  name: string
  description: string
  symbol: string
  timeframe: string
  type: 'TREND_FOLLOWING' | 'MEAN_REVERSION' | 'MOMENTUM' | 'ARBITRAGE' | 'CUSTOM'
  parameters: Record<string, any>
  performance: {
    totalReturn: number
    sharpeRatio: number
    maxDrawdown: number
    winRate: number
    totalTrades: number
  }
  status: 'ACTIVE' | 'INACTIVE' | 'BACKTESTING'
  createdAt: number
  updatedAt: number
}

export interface StrategyRequest {
  symbol: string
  timeframe: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  capital: number
  preferences: {
    maxDrawdown: number
    minWinRate: number
    tradingStyle: 'CONSERVATIVE' | 'MODERATE' | 'AGGRESSIVE'
  }
  constraints: {
    maxPositions: number
    maxLeverage: number
    allowedHours: string[]
  }
}

export interface BacktestResult {
  strategyId: string
  symbol: string
  timeframe: string
  startDate: number
  endDate: number
  initialCapital: number
  finalCapital: number
  totalReturn: number
  annualizedReturn: number
  sharpeRatio: number
  maxDrawdown: number
  winRate: number
  profitFactor: number
  totalTrades: number
  avgTradeDuration: number
  trades: Array<{
    entryTime: number
    exitTime: number
    side: 'LONG' | 'SHORT'
    entryPrice: number
    exitPrice: number
    quantity: number
    pnl: number
    commission: number
  }>
  equity: Array<{
    timestamp: number
    value: number
    drawdown: number
  }>
}

export const useAIStore = defineStore('ai', () => {
  // 状态
  const analysis = ref<Map<string, AIAnalysis>>(new Map())
  const strategies = ref<AIStrategy[]>([])
  const backtestResults = ref<Map<string, BacktestResult>>(new Map())
  const isAnalyzing = ref(false)
  const isGenerating = ref(false)
  const isBacktesting = ref(false)
  const lastError = ref<string | null>(null)
  
  // AI配置
  const aiConfig = ref({
    model: 'deepseek-chat',
    apiKey: '',
    analysisInterval: 300000, // 5分钟
    autoAnalysis: true,
    riskLevel: 'MEDIUM' as const,
    maxStrategies: 10
  })

  // 计算属性
  const activeStrategies = computed(() => {
    return strategies.value.filter(s => s.status === 'ACTIVE')
  })

  const topPerformingStrategies = computed(() => {
    return strategies.value
      .filter(s => s.performance.totalTrades > 0)
      .sort((a, b) => b.performance.totalReturn - a.performance.totalReturn)
      .slice(0, 5)
  })

  const analysisCount = computed(() => analysis.value.size)

  const averageConfidence = computed(() => {
    const analyses = Array.from(analysis.value.values())
    if (analyses.length === 0) return 0
    
    const totalConfidence = analyses.reduce((sum, a) => sum + a.prediction.confidence, 0)
    return totalConfidence / analyses.length
  })

  // 加载AI分析
  const loadAnalysis = async (symbol: string): Promise<AIAnalysis> => {
    try {
      isAnalyzing.value = true
      lastError.value = null

      const response = await axios.get(`/api/ai/analysis/${symbol}`)
      const analysisData = response.data

      analysis.value.set(symbol, analysisData)
      return analysisData
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '加载AI分析失败'
      console.error('Failed to load AI analysis:', error)
      throw error
    } finally {
      isAnalyzing.value = false
    }
  }

  // 刷新分析
  const refreshAnalysis = async (symbol: string): Promise<AIAnalysis> => {
    try {
      isAnalyzing.value = true
      
      const response = await axios.post(`/api/ai/analysis/${symbol}/refresh`)
      const analysisData = response.data

      analysis.value.set(symbol, analysisData)
      return analysisData
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '刷新AI分析失败'
      console.error('Failed to refresh AI analysis:', error)
      throw error
    } finally {
      isAnalyzing.value = false
    }
  }

  // 生成AI策略
  const generateStrategy = async (request: StrategyRequest): Promise<AIStrategy> => {
    try {
      isGenerating.value = true
      lastError.value = null

      const response = await axios.post('/api/ai/strategies/generate', request)
      const strategy = response.data

      strategies.value.unshift(strategy)
      return strategy
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : 'AI策略生成失败'
      console.error('Failed to generate AI strategy:', error)
      throw error
    } finally {
      isGenerating.value = false
    }
  }

  // 加载策略列表
  const loadStrategies = async (): Promise<void> => {
    try {
      const response = await axios.get('/api/ai/strategies')
      strategies.value = response.data
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '加载策略列表失败'
      console.error('Failed to load strategies:', error)
    }
  }

  // 启动策略
  const activateStrategy = async (strategyId: string): Promise<void> => {
    try {
      await axios.post(`/api/ai/strategies/${strategyId}/activate`)
      
      const strategy = strategies.value.find(s => s.id === strategyId)
      if (strategy) {
        strategy.status = 'ACTIVE'
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '启动策略失败'
      console.error('Failed to activate strategy:', error)
      throw error
    }
  }

  // 停止策略
  const deactivateStrategy = async (strategyId: string): Promise<void> => {
    try {
      await axios.post(`/api/ai/strategies/${strategyId}/deactivate`)
      
      const strategy = strategies.value.find(s => s.id === strategyId)
      if (strategy) {
        strategy.status = 'INACTIVE'
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '停止策略失败'
      console.error('Failed to deactivate strategy:', error)
      throw error
    }
  }

  // 删除策略
  const deleteStrategy = async (strategyId: string): Promise<void> => {
    try {
      await axios.delete(`/api/ai/strategies/${strategyId}`)
      
      const index = strategies.value.findIndex(s => s.id === strategyId)
      if (index !== -1) {
        strategies.value.splice(index, 1)
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '删除策略失败'
      console.error('Failed to delete strategy:', error)
      throw error
    }
  }

  // 回测策略
  const backtestStrategy = async (
    strategyId: string, 
    startDate: number, 
    endDate: number,
    initialCapital: number = 10000
  ): Promise<BacktestResult> => {
    try {
      isBacktesting.value = true
      lastError.value = null

      const response = await axios.post(`/api/ai/strategies/${strategyId}/backtest`, {
        startDate,
        endDate,
        initialCapital
      })
      
      const result = response.data
      backtestResults.value.set(strategyId, result)
      
      return result
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '策略回测失败'
      console.error('Failed to backtest strategy:', error)
      throw error
    } finally {
      isBacktesting.value = false
    }
  }

  // 获取市场洞察
  const getMarketInsights = async (symbols: string[]): Promise<any> => {
    try {
      const response = await axios.post('/api/ai/insights', { symbols })
      return response.data
    } catch (error) {
      console.error('Failed to get market insights:', error)
      throw error
    }
  }

  // 获取交易建议
  const getTradingAdvice = async (symbol: string, position?: any): Promise<any> => {
    try {
      const response = await axios.post('/api/ai/advice', { symbol, position })
      return response.data
    } catch (error) {
      console.error('Failed to get trading advice:', error)
      throw error
    }
  }

  // 风险评估
  const assessRisk = async (portfolio: any): Promise<any> => {
    try {
      const response = await axios.post('/api/ai/risk-assessment', { portfolio })
      return response.data
    } catch (error) {
      console.error('Failed to assess risk:', error)
      throw error
    }
  }

  // 优化策略参数
  const optimizeStrategy = async (strategyId: string, parameters: any): Promise<any> => {
    try {
      const response = await axios.post(`/api/ai/strategies/${strategyId}/optimize`, parameters)
      return response.data
    } catch (error) {
      console.error('Failed to optimize strategy:', error)
      throw error
    }
  }

  // 获取AI配置
  const loadAIConfig = async (): Promise<void> => {
    try {
      const response = await axios.get('/api/ai/config')
      aiConfig.value = { ...aiConfig.value, ...response.data }
    } catch (error) {
      console.error('Failed to load AI config:', error)
    }
  }

  // 更新AI配置
  const updateAIConfig = async (config: Partial<typeof aiConfig.value>): Promise<void> => {
    try {
      await axios.put('/api/ai/config', config)
      aiConfig.value = { ...aiConfig.value, ...config }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '更新AI配置失败'
      console.error('Failed to update AI config:', error)
      throw error
    }
  }

  // 获取分析历史
  const getAnalysisHistory = async (symbol: string, limit: number = 50): Promise<AIAnalysis[]> => {
    try {
      const response = await axios.get(`/api/ai/analysis/${symbol}/history`, {
        params: { limit }
      })
      return response.data
    } catch (error) {
      console.error('Failed to get analysis history:', error)
      throw error
    }
  }

  // 导出策略
  const exportStrategy = async (strategyId: string): Promise<Blob> => {
    try {
      const response = await axios.get(`/api/ai/strategies/${strategyId}/export`, {
        responseType: 'blob'
      })
      return response.data
    } catch (error) {
      console.error('Failed to export strategy:', error)
      throw error
    }
  }

  // 导入策略
  const importStrategy = async (file: File): Promise<AIStrategy> => {
    try {
      const formData = new FormData()
      formData.append('strategy', file)
      
      const response = await axios.post('/api/ai/strategies/import', formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      })
      
      const strategy = response.data
      strategies.value.unshift(strategy)
      
      return strategy
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '导入策略失败'
      console.error('Failed to import strategy:', error)
      throw error
    }
  }

  return {
    // 状态
    analysis: readonly(analysis),
    strategies: readonly(strategies),
    backtestResults: readonly(backtestResults),
    isAnalyzing: readonly(isAnalyzing),
    isGenerating: readonly(isGenerating),
    isBacktesting: readonly(isBacktesting),
    lastError: readonly(lastError),
    aiConfig,
    
    // 计算属性
    activeStrategies,
    topPerformingStrategies,
    analysisCount,
    averageConfidence,
    
    // 方法
    loadAnalysis,
    refreshAnalysis,
    generateStrategy,
    loadStrategies,
    activateStrategy,
    deactivateStrategy,
    deleteStrategy,
    backtestStrategy,
    getMarketInsights,
    getTradingAdvice,
    assessRisk,
    optimizeStrategy,
    loadAIConfig,
    updateAIConfig,
    getAnalysisHistory,
    exportStrategy,
    importStrategy
  }
})