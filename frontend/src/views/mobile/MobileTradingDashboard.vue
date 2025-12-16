<template>
  <div class="mobile-trading-dashboard">
    <!-- 顶部状态栏 -->
    <div class="mobile-header">
      <div class="header-left">
        <el-button 
          text 
          @click="showSymbolSelector = true"
          class="symbol-button"
        >
          <span class="symbol">{{ selectedSymbol }}</span>
          <i class="el-icon-arrow-down"></i>
        </el-button>
      </div>
      
      <div class="header-center">
        <div class="price-display">
          <div class="current-price" :class="priceChangeClass">
            {{ formatPrice(currentPrice) }}
          </div>
          <div class="price-change" :class="priceChangeClass">
            {{ formatChange(priceChange) }}
          </div>
        </div>
      </div>
      
      <div class="header-right">
        <el-button text @click="showSettings = true">
          <i class="el-icon-setting"></i>
        </el-button>
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="mobile-content">
      <!-- 标签页导航 -->
      <el-tabs 
        v-model="activeTab" 
        type="border-card"
        class="mobile-tabs"
        @tab-change="onTabChange"
      >
        <!-- 图表页面 -->
        <el-tab-pane label="图表" name="chart">
          <div class="chart-section">
            <!-- 时间周期选择器 -->
            <div class="timeframe-selector">
              <el-button-group size="small">
                <el-button 
                  v-for="tf in timeframes"
                  :key="tf.value"
                  :type="selectedTimeframe === tf.value ? 'primary' : 'default'"
                  @click="changeTimeframe(tf.value)"
                >
                  {{ tf.label }}
                </el-button>
              </el-button-group>
            </div>
            
            <!-- 移动端图表 -->
            <div class="mobile-chart">
              <MobileTradingViewChart
                :symbol="selectedSymbol"
                :interval="selectedTimeframe"
                :height="chartHeight"
                @ready="onChartReady"
              />
            </div>
            
            <!-- 图表控制按钮 -->
            <div class="chart-controls">
              <el-button size="small" @click="toggleIndicators">
                指标
              </el-button>
              <el-button size="small" @click="toggleDrawingTools">
                画线
              </el-button>
              <el-button size="small" @click="toggleFullscreen">
                全屏
              </el-button>
            </div>
          </div>
        </el-tab-pane>

        <!-- 交易页面 -->
        <el-tab-pane label="交易" name="trade">
          <div class="trade-section">
            <!-- 订单簿 -->
            <div class="orderbook-container">
              <MobileOrderBook 
                :symbol="selectedSymbol"
                @price-click="onPriceClick"
              />
            </div>
            
            <!-- 交易面板 -->
            <div class="trading-panel">
              <MobileTradingPanel
                :symbol="selectedSymbol"
                :current-price="currentPrice"
                @order-submit="onOrderSubmit"
              />
            </div>
          </div>
        </el-tab-pane>

        <!-- 持仓页面 -->
        <el-tab-pane label="持仓" name="positions">
          <div class="positions-section">
            <!-- 账户概览 -->
            <div class="account-overview">
              <MobileAccountOverview 
                :balance="accountBalance"
                :pnl="totalPnL"
                :margin="marginInfo"
              />
            </div>
            
            <!-- 持仓列表 -->
            <div class="positions-list">
              <MobilePositionsList
                :positions="userPositions"
                @close="closePosition"
                @modify="modifyPosition"
              />
            </div>
            
            <!-- 订单列表 -->
            <div class="orders-list">
              <MobileOrdersList
                :orders="userOrders"
                @cancel="cancelOrder"
                @modify="modifyOrder"
              />
            </div>
          </div>
        </el-tab-pane>

        <!-- 市场页面 -->
        <el-tab-pane label="市场" name="market">
          <div class="market-section">
            <!-- 搜索栏 -->
            <div class="market-search">
              <el-input
                v-model="searchKeyword"
                placeholder="搜索交易对"
                prefix-icon="el-icon-search"
                @input="onSearchInput"
              />
            </div>
            
            <!-- 市场分类 -->
            <div class="market-categories">
              <el-button-group size="small">
                <el-button 
                  v-for="category in marketCategories"
                  :key="category.value"
                  :type="selectedCategory === category.value ? 'primary' : 'default'"
                  @click="selectCategory(category.value)"
                >
                  {{ category.label }}
                </el-button>
              </el-button-group>
            </div>
            
            <!-- 市场列表 -->
            <div class="market-list">
              <MobileMarketList
                :symbols="filteredSymbols"
                :selected="selectedSymbol"
                @select="onSymbolSelect"
                @favorite="toggleFavorite"
              />
            </div>
          </div>
        </el-tab-pane>

        <!-- AI分析页面 -->
        <el-tab-pane label="AI" name="ai">
          <div class="ai-section">
            <!-- AI分析面板 -->
            <MobileAIAnalysis
              :symbol="selectedSymbol"
              :analysis="aiAnalysis"
              @generate-strategy="generateStrategy"
              @refresh="refreshAIAnalysis"
            />
            
            <!-- 策略列表 -->
            <div class="strategies-list">
              <MobileStrategiesList
                :strategies="userStrategies"
                @toggle="toggleStrategy"
                @edit="editStrategy"
                @delete="deleteStrategy"
              />
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 快速交易浮动按钮 -->
    <div class="quick-trade-fab">
      <el-button 
        type="primary" 
        circle 
        size="large"
        @click="showQuickTrade = true"
      >
        <i class="el-icon-plus"></i>
      </el-button>
    </div>

    <!-- 弹窗组件 -->
    <!-- 交易对选择器 -->
    <el-drawer
      v-model="showSymbolSelector"
      title="选择交易对"
      direction="btt"
      size="80%"
    >
      <MobileSymbolSelector
        :symbols="availableSymbols"
        :selected="selectedSymbol"
        @select="onSymbolSelect"
        @close="showSymbolSelector = false"
      />
    </el-drawer>

    <!-- 快速交易 -->
    <el-drawer
      v-model="showQuickTrade"
      title="快速交易"
      direction="btt"
      size="60%"
    >
      <MobileQuickTrade
        :symbol="selectedSymbol"
        :price="quickTradePrice"
        @submit="onQuickTradeSubmit"
        @close="showQuickTrade = false"
      />
    </el-drawer>

    <!-- 设置面板 -->
    <el-drawer
      v-model="showSettings"
      title="设置"
      direction="rtl"
      size="80%"
    >
      <MobileSettings
        @close="showSettings = false"
      />
    </el-drawer>

    <!-- 指标选择器 -->
    <el-drawer
      v-model="showIndicators"
      title="技术指标"
      direction="btt"
      size="50%"
    >
      <MobileIndicatorSelector
        :selected="selectedIndicators"
        @change="onIndicatorChange"
        @close="showIndicators = false"
      />
    </el-drawer>

    <!-- 画线工具 -->
    <el-drawer
      v-model="showDrawingTools"
      title="画线工具"
      direction="btt"
      size="40%"
    >
      <MobileDrawingTools
        :active="activeDrawingTool"
        @select="onDrawingToolSelect"
        @close="showDrawingTools = false"
      />
    </el-drawer>

    <!-- 全局通知 -->
    <MobileNotifications />
    
    <!-- 连接状态指示器 -->
    <MobileConnectionStatus />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useTradingStore } from '@/stores/trading'
import { useMarketStore } from '@/stores/market'
import { useWebSocketStore } from '@/stores/websocket'
import { useAIStore } from '@/stores/ai'
import { useUserStore } from '@/stores/user'

// 移动端组件导入
import MobileTradingViewChart from '@/components/mobile/MobileTradingViewChart.vue'
import MobileOrderBook from '@/components/mobile/MobileOrderBook.vue'
import MobileTradingPanel from '@/components/mobile/MobileTradingPanel.vue'
import MobileAccountOverview from '@/components/mobile/MobileAccountOverview.vue'
import MobilePositionsList from '@/components/mobile/MobilePositionsList.vue'
import MobileOrdersList from '@/components/mobile/MobileOrdersList.vue'
import MobileMarketList from '@/components/mobile/MobileMarketList.vue'
import MobileAIAnalysis from '@/components/mobile/MobileAIAnalysis.vue'
import MobileStrategiesList from '@/components/mobile/MobileStrategiesList.vue'
import MobileSymbolSelector from '@/components/mobile/MobileSymbolSelector.vue'
import MobileQuickTrade from '@/components/mobile/MobileQuickTrade.vue'
import MobileSettings from '@/components/mobile/MobileSettings.vue'
import MobileIndicatorSelector from '@/components/mobile/MobileIndicatorSelector.vue'
import MobileDrawingTools from '@/components/mobile/MobileDrawingTools.vue'
import MobileNotifications from '@/components/mobile/MobileNotifications.vue'
import MobileConnectionStatus from '@/components/mobile/MobileConnectionStatus.vue'

// 状态管理
const tradingStore = useTradingStore()
const marketStore = useMarketStore()
const wsStore = useWebSocketStore()
const aiStore = useAIStore()
const userStore = useUserStore()
const router = useRouter()

// 响应式数据
const activeTab = ref('chart')
const selectedSymbol = ref('BTCUSDT')
const selectedTimeframe = ref('15m')
const selectedCategory = ref('USDT')
const searchKeyword = ref('')
const currentPrice = ref(0)
const priceChange = ref(0)
const quickTradePrice = ref(0)

// 弹窗状态
const showSymbolSelector = ref(false)
const showQuickTrade = ref(false)
const showSettings = ref(false)
const showIndicators = ref(false)
const showDrawingTools = ref(false)

// 图表相关
const selectedIndicators = ref<string[]>([])
const activeDrawingTool = ref('')

// 时间周期选项
const timeframes = [
  { label: '1m', value: '1m' },
  { label: '5m', value: '5m' },
  { label: '15m', value: '15m' },
  { label: '1H', value: '1h' },
  { label: '4H', value: '4h' },
  { label: '1D', value: '1d' }
]

// 市场分类
const marketCategories = [
  { label: 'USDT', value: 'USDT' },
  { label: 'BTC', value: 'BTC' },
  { label: 'ETH', value: 'ETH' },
  { label: '热门', value: 'HOT' }
]

// 计算属性
const availableSymbols = computed(() => marketStore.symbols)
const userPositions = computed(() => tradingStore.positions)
const userOrders = computed(() => tradingStore.orders)
const userStrategies = computed(() => userStore.strategies)
const accountBalance = computed(() => tradingStore.balance)
const totalPnL = computed(() => tradingStore.totalPnL)
const marginInfo = computed(() => tradingStore.marginInfo)
const aiAnalysis = computed(() => aiStore.analysis[selectedSymbol.value])

const priceChangeClass = computed(() => {
  return priceChange.value >= 0 ? 'positive' : 'negative'
})

const chartHeight = computed(() => {
  const headerHeight = 60
  const tabsHeight = 40
  const controlsHeight = 50
  const windowHeight = window.innerHeight
  return `${windowHeight - headerHeight - tabsHeight - controlsHeight}px`
})

const filteredSymbols = computed(() => {
  let symbols = marketStore.getSymbolsByCategory(selectedCategory.value)
  
  if (searchKeyword.value) {
    symbols = symbols.filter(symbol => 
      symbol.symbol.toLowerCase().includes(searchKeyword.value.toLowerCase())
    )
  }
  
  return symbols
})

// 生命周期
onMounted(async () => {
  await initializeMobileDashboard()
  setupMobileWebSocket()
})

onUnmounted(() => {
  cleanupMobileSubscriptions()
})

// 初始化移动端仪表板
const initializeMobileDashboard = async () => {
  try {
    // 加载市场数据
    await marketStore.loadMarketData()
    
    // 加载用户数据
    await tradingStore.loadUserData()
    
    // 加载AI分析
    await aiStore.loadAnalysis(selectedSymbol.value)
    
    // 设置移动端特定配置
    setupMobileConfig()
    
  } catch (error) {
    console.error('Mobile dashboard initialization failed:', error)
  }
}

// 移动端配置
const setupMobileConfig = () => {
  // 禁用页面缩放
  const viewport = document.querySelector('meta[name=viewport]')
  if (viewport) {
    viewport.setAttribute('content', 'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no')
  }
  
  // 防止页面滚动
  document.body.style.overflow = 'hidden'
  
  // 设置状态栏颜色
  const themeColor = document.querySelector('meta[name=theme-color]')
  if (themeColor) {
    themeColor.setAttribute('content', '#0a0e1a')
  }
}

// WebSocket设置
const setupMobileWebSocket = () => {
  // 订阅价格数据
  wsStore.subscribe('ticker', selectedSymbol.value, (data) => {
    currentPrice.value = data.price
    priceChange.value = data.change
  })
  
  // 订阅用户数据
  wsStore.subscribe('user_data', null, (data) => {
    tradingStore.updateUserData(data)
  })
}

// 事件处理
const onTabChange = (tabName: string) => {
  activeTab.value = tabName
  
  // 根据标签页加载相应数据
  switch (tabName) {
    case 'chart':
      // 图表页面无需额外加载
      break
    case 'trade':
      // 加载订单簿数据
      marketStore.loadOrderBook(selectedSymbol.value)
      break
    case 'positions':
      // 刷新持仓数据
      tradingStore.refreshPositions()
      break
    case 'market':
      // 刷新市场数据
      marketStore.refreshMarketData()
      break
    case 'ai':
      // 刷新AI分析
      aiStore.refreshAnalysis(selectedSymbol.value)
      break
  }
}

const onSymbolSelect = (symbol: string) => {
  selectedSymbol.value = symbol
  showSymbolSelector.value = false
  
  // 重新订阅新交易对数据
  setupMobileWebSocket()
  
  // 加载AI分析
  aiStore.loadAnalysis(symbol)
}

const changeTimeframe = (timeframe: string) => {
  selectedTimeframe.value = timeframe
}

const onChartReady = (chart: any) => {
  console.log('Mobile chart ready:', chart)
}

const onPriceClick = (price: number) => {
  quickTradePrice.value = price
  showQuickTrade.value = true
}

const onOrderSubmit = async (order: any) => {
  try {
    await tradingStore.submitOrder(order)
    ElMessage.success('订单提交成功')
  } catch (error) {
    ElMessage.error('订单提交失败: ' + error.message)
  }
}

const onQuickTradeSubmit = async (order: any) => {
  await onOrderSubmit(order)
  showQuickTrade.value = false
}

const selectCategory = (category: string) => {
  selectedCategory.value = category
}

const onSearchInput = (keyword: string) => {
  searchKeyword.value = keyword
}

const toggleFavorite = (symbol: string) => {
  userStore.toggleWatchlist(symbol)
}

// 图表控制
const toggleIndicators = () => {
  showIndicators.value = true
}

const toggleDrawingTools = () => {
  showDrawingTools.value = true
}

const toggleFullscreen = () => {
  if (document.documentElement.requestFullscreen) {
    document.documentElement.requestFullscreen()
  }
}

const onIndicatorChange = (indicators: string[]) => {
  selectedIndicators.value = indicators
  showIndicators.value = false
}

const onDrawingToolSelect = (tool: string) => {
  activeDrawingTool.value = tool
  showDrawingTools.value = false
}

// AI相关
const generateStrategy = async () => {
  try {
    await aiStore.generateStrategy({
      symbol: selectedSymbol.value,
      timeframe: selectedTimeframe.value
    })
    ElMessage.success('AI策略生成成功')
  } catch (error) {
    ElMessage.error('AI策略生成失败: ' + error.message)
  }
}

const refreshAIAnalysis = async () => {
  try {
    await aiStore.refreshAnalysis(selectedSymbol.value)
    ElMessage.success('AI分析已刷新')
  } catch (error) {
    ElMessage.error('AI分析刷新失败: ' + error.message)
  }
}

// 策略管理
const toggleStrategy = async (strategyId: string) => {
  try {
    await tradingStore.toggleStrategy(strategyId)
  } catch (error) {
    ElMessage.error('策略切换失败: ' + error.message)
  }
}

const editStrategy = (strategyId: string) => {
  router.push(`/strategy/edit/${strategyId}`)
}

const deleteStrategy = async (strategyId: string) => {
  try {
    await tradingStore.deleteStrategy(strategyId)
    ElMessage.success('策略删除成功')
  } catch (error) {
    ElMessage.error('策略删除失败: ' + error.message)
  }
}

// 持仓和订单管理
const closePosition = async (positionId: string) => {
  try {
    await tradingStore.closePosition(positionId)
    ElMessage.success('平仓成功')
  } catch (error) {
    ElMessage.error('平仓失败: ' + error.message)
  }
}

const modifyPosition = (positionId: string) => {
  // 修改持仓逻辑
}

const cancelOrder = async (orderId: string) => {
  try {
    await tradingStore.cancelOrder(orderId)
    ElMessage.success('订单取消成功')
  } catch (error) {
    ElMessage.error('订单取消失败: ' + error.message)
  }
}

const modifyOrder = (orderId: string) => {
  // 修改订单逻辑
}

// 工具函数
const formatPrice = (price: number) => {
  return price.toFixed(2)
}

const formatChange = (change: number) => {
  const sign = change >= 0 ? '+' : ''
  return `${sign}${change.toFixed(2)}%`
}

// 清理资源
const cleanupMobileSubscriptions = () => {
  wsStore.unsubscribeAll()
  document.body.style.overflow = 'auto'
}

// 监听交易对变化
watch(selectedSymbol, (newSymbol) => {
  setupMobileWebSocket()
  aiStore.loadAnalysis(newSymbol)
})
</script>

<style lang="scss" scoped>
.mobile-trading-dashboard {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
}

.mobile-header {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  
  .header-left {
    .symbol-button {
      .symbol {
        font-weight: 600;
        font-size: 16px;
      }
    }
  }
  
  .header-center {
    .price-display {
      text-align: center;
      
      .current-price {
        font-size: 20px;
        font-weight: 700;
        line-height: 1;
        
        &.positive {
          color: var(--success-color);
        }
        
        &.negative {
          color: var(--error-color);
        }
      }
      
      .price-change {
        font-size: 12px;
        margin-top: 2px;
        
        &.positive {
          color: var(--success-color);
        }
        
        &.negative {
          color: var(--error-color);
        }
      }
    }
  }
}

.mobile-content {
  flex: 1;
  overflow: hidden;
  
  .mobile-tabs {
    height: 100%;
    
    :deep(.el-tabs__header) {
      margin: 0;
      
      .el-tabs__nav-wrap {
        padding: 0 16px;
      }
    }
    
    :deep(.el-tabs__content) {
      height: calc(100% - 40px);
      overflow: hidden;
    }
    
    :deep(.el-tab-pane) {
      height: 100%;
      overflow-y: auto;
    }
  }
}

.chart-section {
  height: 100%;
  display: flex;
  flex-direction: column;
  
  .timeframe-selector {
    padding: 12px 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .mobile-chart {
    flex: 1;
    overflow: hidden;
  }
  
  .chart-controls {
    height: 50px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
  }
}

.trade-section {
  height: 100%;
  display: flex;
  flex-direction: column;
  
  .orderbook-container {
    height: 300px;
    border-bottom: 1px solid var(--border-color);
  }
  
  .trading-panel {
    flex: 1;
    overflow-y: auto;
  }
}

.positions-section {
  height: 100%;
  overflow-y: auto;
  
  .account-overview {
    padding: 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .positions-list,
  .orders-list {
    margin-bottom: 16px;
  }
}

.market-section {
  height: 100%;
  display: flex;
  flex-direction: column;
  
  .market-search {
    padding: 12px 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .market-categories {
    padding: 12px 16px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .market-list {
    flex: 1;
    overflow-y: auto;
  }
}

.ai-section {
  height: 100%;
  overflow-y: auto;
  
  .strategies-list {
    margin-top: 16px;
  }
}

.quick-trade-fab {
  position: fixed;
  bottom: 80px;
  right: 20px;
  z-index: 1000;
  
  .el-button {
    width: 56px;
    height: 56px;
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
  }
}

// 抽屉样式覆盖
:deep(.el-drawer) {
  .el-drawer__header {
    padding: 16px;
    border-bottom: 1px solid var(--border-color);
    
    .el-drawer__title {
      font-size: 18px;
      font-weight: 600;
    }
  }
  
  .el-drawer__body {
    padding: 0;
  }
}

// 响应式适配
@media (max-width: 375px) {
  .mobile-header {
    padding: 0 12px;
    
    .header-center {
      .price-display {
        .current-price {
          font-size: 18px;
        }
      }
    }
  }
  
  .quick-trade-fab {
    bottom: 70px;
    right: 16px;
    
    .el-button {
      width: 48px;
      height: 48px;
    }
  }
}

// 横屏适配
@media (orientation: landscape) and (max-height: 500px) {
  .mobile-header {
    height: 50px;
  }
  
  .chart-section {
    .timeframe-selector {
      padding: 8px 16px;
    }
    
    .chart-controls {
      height: 40px;
    }
  }
  
  .quick-trade-fab {
    bottom: 60px;
  }
}
</style>