<template>
  <div class="orderbook-container">
    <!-- 订单簿头部 -->
    <div class="orderbook-header">
      <div class="header-left">
        <span class="title">订单簿</span>
        <el-tag size="small" :type="spreadType">
          价差: {{ formatSpread(spread) }}
        </el-tag>
      </div>
      <div class="header-right">
        <el-dropdown @command="handlePrecisionChange">
          <el-button size="small" text>
            {{ currentPrecision }}位 <i class="el-icon-arrow-down"></i>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item 
                v-for="p in precisionOptions" 
                :key="p"
                :command="p"
              >
                {{ p }}位小数
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        
        <el-button size="small" text @click="toggleDepth">
          <i :class="depthIcon"></i>
        </el-button>
      </div>
    </div>

    <!-- 订单簿内容 -->
    <div class="orderbook-content" ref="orderbookRef">
      <!-- 卖单区域 -->
      <div class="asks-section">
        <div class="section-header">
          <span class="price-header">价格({{ quoteAsset }})</span>
          <span class="amount-header">数量({{ baseAsset }})</span>
          <span class="total-header">累计</span>
        </div>
        
        <div class="asks-list" ref="asksRef">
          <div 
            v-for="(ask, index) in displayAsks" 
            :key="`ask-${index}`"
            class="orderbook-row ask-row"
            :class="{ 'highlighted': highlightedPrice === ask.price }"
            @click="onPriceClick(ask.price, 'sell')"
            @mouseenter="onRowHover(ask.price)"
            @mouseleave="onRowLeave"
          >
            <!-- 深度背景条 -->
            <div 
              class="depth-bar ask-depth"
              :style="{ width: `${ask.depthPercentage}%` }"
            ></div>
            
            <!-- 价格 -->
            <span class="price ask-price">
              {{ formatPrice(ask.price) }}
            </span>
            
            <!-- 数量 -->
            <span class="amount">
              {{ formatAmount(ask.amount) }}
            </span>
            
            <!-- 累计 -->
            <span class="total">
              {{ formatAmount(ask.total) }}
            </span>
            
            <!-- 闪烁效果 -->
            <div 
              v-if="ask.isUpdated" 
              class="flash-overlay"
              :class="ask.changeType"
            ></div>
          </div>
        </div>
      </div>

      <!-- 中间价格区域 -->
      <div class="price-spread-section">
        <div class="current-price" :class="priceChangeClass">
          <div class="price-value">
            {{ formatPrice(currentPrice) }}
          </div>
          <div class="price-change">
            <i :class="priceChangeIcon"></i>
            {{ formatPriceChange(priceChange) }}
          </div>
        </div>
        
        <div class="spread-info">
          <span class="spread-label">价差</span>
          <span class="spread-value" :class="spreadType">
            {{ formatSpread(spread) }}
          </span>
          <span class="spread-percentage">
            ({{ formatPercentage(spreadPercentage) }})
          </span>
        </div>
      </div>

      <!-- 买单区域 -->
      <div class="bids-section">
        <div class="bids-list" ref="bidsRef">
          <div 
            v-for="(bid, index) in displayBids" 
            :key="`bid-${index}`"
            class="orderbook-row bid-row"
            :class="{ 'highlighted': highlightedPrice === bid.price }"
            @click="onPriceClick(bid.price, 'buy')"
            @mouseenter="onRowHover(bid.price)"
            @mouseleave="onRowLeave"
          >
            <!-- 深度背景条 -->
            <div 
              class="depth-bar bid-depth"
              :style="{ width: `${bid.depthPercentage}%` }"
            ></div>
            
            <!-- 价格 -->
            <span class="price bid-price">
              {{ formatPrice(bid.price) }}
            </span>
            
            <!-- 数量 -->
            <span class="amount">
              {{ formatAmount(bid.amount) }}
            </span>
            
            <!-- 累计 -->
            <span class="total">
              {{ formatAmount(bid.total) }}
            </span>
            
            <!-- 闪烁效果 -->
            <div 
              v-if="bid.isUpdated" 
              class="flash-overlay"
              :class="bid.changeType"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <!-- 订单簿统计 -->
    <div class="orderbook-stats">
      <div class="stat-item">
        <span class="stat-label">买单总量</span>
        <span class="stat-value bid-color">{{ formatAmount(totalBidAmount) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">卖单总量</span>
        <span class="stat-value ask-color">{{ formatAmount(totalAskAmount) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">买卖比</span>
        <span class="stat-value" :class="ratioClass">{{ formatRatio(bidAskRatio) }}</span>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="orderbook-loading">
      <i class="el-icon-loading"></i>
      <span>加载中...</span>
    </div>

    <!-- 错误状态 -->
    <div v-if="hasError" class="orderbook-error">
      <i class="el-icon-warning"></i>
      <span>数据加载失败</span>
      <el-button size="small" @click="retry">重试</el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useWebSocketStore } from '@/stores/websocket'
import { useMarketStore } from '@/stores/market'
import { useTradingStore } from '@/stores/trading'
import Decimal from 'decimal.js'

// Props
interface Props {
  symbol: string
  precision?: number
  maxDepth?: number
  showStats?: boolean
  clickable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  precision: 2,
  maxDepth: 20,
  showStats: true,
  clickable: true
})

// Emits
const emit = defineEmits<{
  priceClick: [price: number, side: 'buy' | 'sell']
  depthChange: [depth: number]
}>()

// 响应式数据
const orderbookRef = ref<HTMLElement>()
const asksRef = ref<HTMLElement>()
const bidsRef = ref<HTMLElement>()

const currentPrecision = ref(props.precision)
const currentDepth = ref(props.maxDepth)
const highlightedPrice = ref<number | null>(null)
const isLoading = ref(true)
const hasError = ref(false)

// 订单簿数据
const rawOrderbook = ref<any>({
  bids: [],
  asks: [],
  lastUpdateId: 0
})

const currentPrice = ref(0)
const priceChange = ref(0)

// 状态管理
const wsStore = useWebSocketStore()
const marketStore = useMarketStore()
const tradingStore = useTradingStore()

// 精度选项
const precisionOptions = [0, 1, 2, 3, 4, 5, 6, 7, 8]

// 计算属性
const baseAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[0] || props.symbol.substring(0, 3)
})

const quoteAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[1] || props.symbol.substring(3)
})

// 处理订单簿数据
const processedOrderbook = computed(() => {
  if (!rawOrderbook.value.bids || !rawOrderbook.value.asks) {
    return { bids: [], asks: [] }
  }

  // 合并相同价格的订单（根据精度）
  const groupedBids = groupByPrecision(rawOrderbook.value.bids, currentPrecision.value)
  const groupedAsks = groupByPrecision(rawOrderbook.value.asks, currentPrecision.value)

  // 计算累计数量和深度百分比
  const processedBids = calculateDepth(groupedBids, 'bid')
  const processedAsks = calculateDepth(groupedAsks, 'ask')

  return {
    bids: processedBids.slice(0, currentDepth.value),
    asks: processedAsks.slice(0, currentDepth.value)
  }
})

const displayBids = computed(() => processedOrderbook.value.bids)
const displayAsks = computed(() => processedOrderbook.value.asks.reverse())

// 价格相关计算
const bestBid = computed(() => {
  return displayBids.value[0]?.price || 0
})

const bestAsk = computed(() => {
  return displayAsks.value[displayAsks.value.length - 1]?.price || 0
})

const spread = computed(() => {
  return bestAsk.value - bestBid.value
})

const spreadPercentage = computed(() => {
  if (bestBid.value === 0) return 0
  return (spread.value / bestBid.value) * 100
})

const spreadType = computed(() => {
  if (spreadPercentage.value < 0.1) return 'success'
  if (spreadPercentage.value < 0.5) return 'warning'
  return 'danger'
})

const priceChangeClass = computed(() => {
  return priceChange.value >= 0 ? 'positive' : 'negative'
})

const priceChangeIcon = computed(() => {
  return priceChange.value >= 0 ? 'el-icon-caret-top' : 'el-icon-caret-bottom'
})

const depthIcon = computed(() => {
  return currentDepth.value === props.maxDepth ? 'el-icon-minus' : 'el-icon-plus'
})

// 统计数据
const totalBidAmount = computed(() => {
  return displayBids.value.reduce((sum, bid) => sum + bid.amount, 0)
})

const totalAskAmount = computed(() => {
  return displayAsks.value.reduce((sum, ask) => sum + ask.amount, 0)
})

const bidAskRatio = computed(() => {
  if (totalAskAmount.value === 0) return 0
  return totalBidAmount.value / totalAskAmount.value
})

const ratioClass = computed(() => {
  if (bidAskRatio.value > 1.2) return 'bid-color'
  if (bidAskRatio.value < 0.8) return 'ask-color'
  return ''
})

// 生命周期
onMounted(() => {
  initOrderbook()
  setupWebSocketSubscription()
})

onUnmounted(() => {
  cleanupSubscription()
})

// 初始化订单簿
const initOrderbook = async () => {
  try {
    isLoading.value = true
    hasError.value = false
    
    // 获取初始订单簿数据
    const initialData = await marketStore.getOrderBook(props.symbol, currentDepth.value)
    rawOrderbook.value = initialData
    
    // 获取当前价格
    const ticker = await marketStore.getTicker(props.symbol)
    currentPrice.value = ticker.lastPrice
    priceChange.value = ticker.priceChangePercent
    
  } catch (error) {
    console.error('Failed to initialize orderbook:', error)
    hasError.value = true
  } finally {
    isLoading.value = false
  }
}

// WebSocket订阅
const setupWebSocketSubscription = () => {
  // 订阅订单簿更新
  wsStore.subscribe('depth', props.symbol, (data) => {
    updateOrderbook(data)
  })
  
  // 订阅价格更新
  wsStore.subscribe('ticker', props.symbol, (data) => {
    currentPrice.value = data.lastPrice
    priceChange.value = data.priceChangePercent
  })
}

// 更新订单簿数据
const updateOrderbook = (data: any) => {
  try {
    // 检查更新ID，确保数据顺序
    if (data.lastUpdateId <= rawOrderbook.value.lastUpdateId) {
      return
    }

    // 应用增量更新
    applyDepthUpdate(data)
    
    // 添加闪烁效果
    addFlashEffect(data)
    
    rawOrderbook.value.lastUpdateId = data.lastUpdateId
    
  } catch (error) {
    console.error('Failed to update orderbook:', error)
  }
}

// 应用深度更新
const applyDepthUpdate = (data: any) => {
  // 更新买单
  if (data.bids) {
    data.bids.forEach(([price, amount]: [string, string]) => {
      const priceNum = parseFloat(price)
      const amountNum = parseFloat(amount)
      
      const index = rawOrderbook.value.bids.findIndex((bid: any) => bid[0] === price)
      
      if (amountNum === 0) {
        // 删除订单
        if (index !== -1) {
          rawOrderbook.value.bids.splice(index, 1)
        }
      } else {
        // 更新或添加订单
        if (index !== -1) {
          rawOrderbook.value.bids[index] = [price, amount]
        } else {
          rawOrderbook.value.bids.push([price, amount])
        }
      }
    })
    
    // 重新排序（价格从高到低）
    rawOrderbook.value.bids.sort((a: any, b: any) => parseFloat(b[0]) - parseFloat(a[0]))
  }
  
  // 更新卖单
  if (data.asks) {
    data.asks.forEach(([price, amount]: [string, string]) => {
      const priceNum = parseFloat(price)
      const amountNum = parseFloat(amount)
      
      const index = rawOrderbook.value.asks.findIndex((ask: any) => ask[0] === price)
      
      if (amountNum === 0) {
        // 删除订单
        if (index !== -1) {
          rawOrderbook.value.asks.splice(index, 1)
        }
      } else {
        // 更新或添加订单
        if (index !== -1) {
          rawOrderbook.value.asks[index] = [price, amount]
        } else {
          rawOrderbook.value.asks.push([price, amount])
        }
      }
    })
    
    // 重新排序（价格从低到高）
    rawOrderbook.value.asks.sort((a: any, b: any) => parseFloat(a[0]) - parseFloat(b[0]))
  }
}

// 添加闪烁效果
const addFlashEffect = (data: any) => {
  // 实现价格变化的视觉反馈
  nextTick(() => {
    // 添加闪烁动画类
    const updatedPrices = new Set()
    
    if (data.bids) {
      data.bids.forEach(([price]: [string, string]) => {
        updatedPrices.add(price)
      })
    }
    
    if (data.asks) {
      data.asks.forEach(([price]: [string, string]) => {
        updatedPrices.add(price)
      })
    }
    
    // 触发闪烁动画
    updatedPrices.forEach(price => {
      const element = document.querySelector(`[data-price="${price}"]`)
      if (element) {
        element.classList.add('flash-update')
        setTimeout(() => {
          element.classList.remove('flash-update')
        }, 300)
      }
    })
  })
}

// 根据精度分组
const groupByPrecision = (orders: any[], precision: number) => {
  const grouped = new Map()
  
  orders.forEach(([price, amount]) => {
    const roundedPrice = new Decimal(price).toFixed(precision)
    const currentAmount = grouped.get(roundedPrice) || 0
    grouped.set(roundedPrice, currentAmount + parseFloat(amount))
  })
  
  return Array.from(grouped.entries()).map(([price, amount]) => ({
    price: parseFloat(price),
    amount: amount
  }))
}

// 计算深度
const calculateDepth = (orders: any[], type: 'bid' | 'ask') => {
  let total = 0
  const maxAmount = Math.max(...orders.map(order => order.amount))
  
  return orders.map(order => {
    total += order.amount
    return {
      ...order,
      total,
      depthPercentage: (order.amount / maxAmount) * 100,
      isUpdated: false,
      changeType: ''
    }
  })
}

// 事件处理
const onPriceClick = (price: number, side: 'buy' | 'sell') => {
  if (!props.clickable) return
  
  emit('priceClick', price, side)
  
  // 添加点击反馈
  highlightedPrice.value = price
  setTimeout(() => {
    highlightedPrice.value = null
  }, 500)
}

const onRowHover = (price: number) => {
  // 悬停效果
}

const onRowLeave = () => {
  // 离开悬停
}

const handlePrecisionChange = (precision: number) => {
  currentPrecision.value = precision
}

const toggleDepth = () => {
  currentDepth.value = currentDepth.value === props.maxDepth ? 10 : props.maxDepth
  emit('depthChange', currentDepth.value)
}

const retry = () => {
  initOrderbook()
}

const cleanupSubscription = () => {
  wsStore.unsubscribe('depth', props.symbol)
  wsStore.unsubscribe('ticker', props.symbol)
}

// 格式化函数
const formatPrice = (price: number) => {
  return new Decimal(price).toFixed(currentPrecision.value)
}

const formatAmount = (amount: number) => {
  if (amount >= 1000000) {
    return (amount / 1000000).toFixed(2) + 'M'
  } else if (amount >= 1000) {
    return (amount / 1000).toFixed(2) + 'K'
  }
  return amount.toFixed(4)
}

const formatSpread = (spread: number) => {
  return new Decimal(spread).toFixed(currentPrecision.value)
}

const formatPercentage = (percentage: number) => {
  return percentage.toFixed(3) + '%'
}

const formatPriceChange = (change: number) => {
  const sign = change >= 0 ? '+' : ''
  return `${sign}${change.toFixed(2)}%`
}

const formatRatio = (ratio: number) => {
  return ratio.toFixed(2)
}

// 监听属性变化
watch(() => props.symbol, (newSymbol) => {
  cleanupSubscription()
  initOrderbook()
  setupWebSocketSubscription()
})

watch(currentPrecision, () => {
  // 精度变化时重新计算显示
})

watch(currentDepth, () => {
  // 深度变化时重新获取数据
  initOrderbook()
})
</script>

<style lang="scss" scoped>
.orderbook-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

.orderbook-header {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  
  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    
    .title {
      font-size: 14px;
      font-weight: 600;
      color: var(--text-primary);
    }
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }
}

.orderbook-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.section-header {
  height: 24px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  
  .price-header {
    flex: 1;
    text-align: right;
  }
  
  .amount-header {
    flex: 1;
    text-align: right;
  }
  
  .total-header {
    flex: 1;
    text-align: right;
  }
}

.asks-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  
  .asks-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column-reverse; // 卖单从下往上显示
  }
}

.bids-section {
  flex: 1;
  
  .bids-list {
    height: 100%;
    overflow-y: auto;
  }
}

.orderbook-row {
  height: 20px;
  display: flex;
  align-items: center;
  padding: 0 8px;
  position: relative;
  cursor: pointer;
  font-size: 12px;
  transition: background-color 0.2s ease;
  
  &:hover {
    background: var(--bg-tertiary);
  }
  
  &.highlighted {
    background: var(--accent-primary) !important;
    color: white;
  }
  
  .price {
    flex: 1;
    text-align: right;
    font-weight: 600;
    
    &.bid-price {
      color: var(--success-color);
    }
    
    &.ask-price {
      color: var(--error-color);
    }
  }
  
  .amount {
    flex: 1;
    text-align: right;
    color: var(--text-primary);
  }
  
  .total {
    flex: 1;
    text-align: right;
    color: var(--text-secondary);
    font-size: 11px;
  }
}

.depth-bar {
  position: absolute;
  top: 0;
  right: 0;
  height: 100%;
  opacity: 0.3;
  transition: width 0.3s ease;
  
  &.bid-depth {
    background: var(--success-color);
  }
  
  &.ask-depth {
    background: var(--error-color);
  }
}

.flash-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  animation: flash 0.3s ease-out;
  
  &.increase {
    background: var(--success-color);
  }
  
  &.decrease {
    background: var(--error-color);
  }
}

.price-spread-section {
  height: 60px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  margin: 2px 0;
  
  .current-price {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 4px;
    
    .price-value {
      font-size: 16px;
      font-weight: 700;
    }
    
    .price-change {
      display: flex;
      align-items: center;
      gap: 2px;
      font-size: 12px;
    }
    
    &.positive {
      color: var(--success-color);
    }
    
    &.negative {
      color: var(--error-color);
    }
  }
  
  .spread-info {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--text-secondary);
    
    .spread-value {
      font-weight: 600;
      
      &.success {
        color: var(--success-color);
      }
      
      &.warning {
        color: var(--warning-color);
      }
      
      &.danger {
        color: var(--error-color);
      }
    }
  }
}

.orderbook-stats {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-around;
  padding: 0 8px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  
  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    
    .stat-label {
      color: var(--text-secondary);
    }
    
    .stat-value {
      font-weight: 600;
      
      &.bid-color {
        color: var(--success-color);
      }
      
      &.ask-color {
        color: var(--error-color);
      }
    }
  }
}

.orderbook-loading,
.orderbook-error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}

// 动画效果
@keyframes flash {
  0% {
    opacity: 0.8;
  }
  100% {
    opacity: 0;
  }
}

.flash-update {
  animation: flash 0.3s ease-out;
}

// 滚动条样式
.asks-list,
.bids-list {
  &::-webkit-scrollbar {
    width: 4px;
  }
  
  &::-webkit-scrollbar-track {
    background: transparent;
  }
  
  &::-webkit-scrollbar-thumb {
    background: var(--border-color);
    border-radius: 2px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .orderbook-row {
    height: 24px;
    font-size: 11px;
  }
  
  .section-header {
    font-size: 10px;
  }
  
  .price-spread-section {
    height: 50px;
    
    .current-price {
      .price-value {
        font-size: 14px;
      }
    }
  }
}
</style>