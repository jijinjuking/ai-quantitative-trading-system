<template>
  <div class="trading-panel">
    <!-- 交易类型选择 -->
    <div class="trading-tabs">
      <el-tabs v-model="activeTab" type="border-card">
        <el-tab-pane label="现货" name="spot">
          <SpotTradingForm 
            :symbol="symbol"
            :current-price="currentPrice"
            @order-submit="onOrderSubmit"
          />
        </el-tab-pane>
        <el-tab-pane label="合约" name="futures">
          <FuturesTradingForm 
            :symbol="symbol"
            :current-price="currentPrice"
            @order-submit="onOrderSubmit"
          />
        </el-tab-pane>
        <el-tab-pane label="期权" name="options" v-if="isVIP">
          <OptionsTradingForm 
            :symbol="symbol"
            :current-price="currentPrice"
            @order-submit="onOrderSubmit"
          />
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 快速交易按钮 -->
    <div class="quick-actions" v-if="showQuickActions">
      <div class="quick-buttons">
        <el-button 
          type="success" 
          size="large"
          :loading="isSubmitting"
          @click="quickBuy"
          class="quick-buy"
        >
          <i class="el-icon-caret-top"></i>
          快速买入
        </el-button>
        <el-button 
          type="danger" 
          size="large"
          :loading="isSubmitting"
          @click="quickSell"
          class="quick-sell"
        >
          <i class="el-icon-caret-bottom"></i>
          快速卖出
        </el-button>
      </div>
      
      <div class="quick-amount">
        <el-input-number
          v-model="quickAmount"
          :min="0"
          :precision="quantityPrecision"
          size="small"
          placeholder="数量"
        />
      </div>
    </div>

    <!-- 账户信息 -->
    <div class="account-info">
      <div class="balance-section">
        <div class="balance-item">
          <span class="label">可用余额</span>
          <span class="value">{{ formatBalance(availableBalance) }} {{ quoteAsset }}</span>
        </div>
        <div class="balance-item" v-if="activeTab === 'futures'">
          <span class="label">保证金</span>
          <span class="value">{{ formatBalance(marginBalance) }} {{ quoteAsset }}</span>
        </div>
        <div class="balance-item" v-if="activeTab === 'futures'">
          <span class="label">杠杆</span>
          <div class="leverage-control">
            <el-slider
              v-model="leverage"
              :min="1"
              :max="maxLeverage"
              :step="1"
              @change="onLeverageChange"
            />
            <span class="leverage-value">{{ leverage }}x</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 持仓信息 -->
    <div class="position-info" v-if="currentPosition && activeTab === 'futures'">
      <div class="position-header">
        <h4>当前持仓</h4>
        <el-button size="small" type="danger" @click="closePosition">
          平仓
        </el-button>
      </div>
      <div class="position-details">
        <div class="position-item">
          <span class="label">方向</span>
          <span class="value" :class="currentPosition.side.toLowerCase()">
            {{ currentPosition.side === 'LONG' ? '多头' : '空头' }}
          </span>
        </div>
        <div class="position-item">
          <span class="label">数量</span>
          <span class="value">{{ formatAmount(currentPosition.size) }}</span>
        </div>
        <div class="position-item">
          <span class="label">开仓价</span>
          <span class="value">{{ formatPrice(currentPosition.entryPrice) }}</span>
        </div>
        <div class="position-item">
          <span class="label">未实现盈亏</span>
          <span class="value" :class="currentPosition.unrealizedPnl >= 0 ? 'positive' : 'negative'">
            {{ formatPnL(currentPosition.unrealizedPnl) }}
          </span>
        </div>
      </div>
    </div>

    <!-- 风险提示 -->
    <div class="risk-warning" v-if="showRiskWarning">
      <el-alert
        :title="riskWarningText"
        type="warning"
        :closable="false"
        show-icon
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useTradingStore } from '@/stores/trading'
import { useMarketStore } from '@/stores/market'
import { useUserStore } from '@/stores/user'
import SpotTradingForm from './SpotTradingForm.vue'
import FuturesTradingForm from './FuturesTradingForm.vue'
import OptionsTradingForm from './OptionsTradingForm.vue'

// Props
interface Props {
  symbol: string
  currentPrice: number
  showQuickActions?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showQuickActions: true
})

// Emits
const emit = defineEmits<{
  orderSubmit: [order: any]
}>()

// 状态管理
const tradingStore = useTradingStore()
const marketStore = useMarketStore()
const userStore = useUserStore()

// 响应式数据
const activeTab = ref('spot')
const isSubmitting = ref(false)
const quickAmount = ref(0)
const leverage = ref(10)

// 计算属性
const baseAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[0] || props.symbol.substring(0, 3)
})

const quoteAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[1] || props.symbol.substring(3)
})

const availableBalance = computed(() => {
  const balance = tradingStore.assets.find(a => a.asset === quoteAsset.value)
  return balance?.free || 0
})

const marginBalance = computed(() => {
  return tradingStore.marginInfo?.availableMargin || 0
})

const maxLeverage = computed(() => {
  return userStore.user?.kyc.limits.maxLeverage || 20
})

const currentPosition = computed(() => {
  return tradingStore.activePositions.find(p => p.symbol === props.symbol)
})

const isVIP = computed(() => userStore.isVIP)

const quantityPrecision = computed(() => {
  return marketStore.getQuantityPrecision(props.symbol)
})

const pricePrecision = computed(() => {
  return marketStore.getPricePrecision(props.symbol)
})

const showRiskWarning = computed(() => {
  return activeTab.value === 'futures' && leverage.value > 10
})

const riskWarningText = computed(() => {
  if (leverage.value > 20) {
    return '极高杠杆风险：可能导致快速爆仓，请谨慎操作'
  } else if (leverage.value > 10) {
    return '高杠杆风险：请注意风险控制，设置止损'
  }
  return ''
})

// 生命周期
onMounted(() => {
  loadTradingData()
})

// 加载交易数据
const loadTradingData = async () => {
  try {
    await tradingStore.loadUserData()
  } catch (error) {
    console.error('Failed to load trading data:', error)
  }
}

// 事件处理
const onOrderSubmit = async (order: any) => {
  try {
    isSubmitting.value = true
    
    // 验证订单
    const validation = tradingStore.validateOrder(order)
    if (!validation.valid) {
      ElMessage.error(validation.error)
      return
    }
    
    // 风险确认
    if (userStore.user?.preferences.trading.confirmOrders) {
      const confirmed = await showOrderConfirmation(order)
      if (!confirmed) return
    }
    
    // 提交订单
    const result = await tradingStore.submitOrder(order)
    
    emit('orderSubmit', result)
    // ElMessage.success('订单提交成功')
    
    // 播放提示音
    if (userStore.user?.preferences?.trading?.soundEnabled) {
      playOrderSound()
    }
    
  } catch (error: any) {
    // ElMessage.error('订单提交失败: ' + error.message)
    console.error('订单提交失败:', error)
  } finally {
    isSubmitting.value = false
  }
}

const quickBuy = async () => {
  if (quickAmount.value <= 0) {
    // ElMessage.warning('请输入有效数量')
    console.warn('请输入有效数量')
    return
  }
  
  const order = {
    symbol: props.symbol,
    side: 'BUY' as const,
    type: 'MARKET' as const,
    quantity: quickAmount.value
  }
  
  await onOrderSubmit(order)
}

const quickSell = async () => {
  if (quickAmount.value <= 0) {
    // ElMessage.warning('请输入有效数量')
    console.warn('请输入有效数量')
    return
  }
  
  const order = {
    symbol: props.symbol,
    side: 'SELL' as const,
    type: 'MARKET' as const,
    quantity: quickAmount.value
  }
  
  await onOrderSubmit(order)
}

const closePosition = async () => {
  if (!currentPosition.value) return
  
  try {
    // await ElMessageBox.confirm(
    //   '确定要平仓当前持仓吗？',
    //   '平仓确认',
    //   {
    //     confirmButtonText: '确定',
    //     cancelButtonText: '取消',
    //     type: 'warning'
    //   }
    // )
    
    await tradingStore.closePosition(currentPosition.value.id)
    // ElMessage.success('平仓成功')
    console.log('平仓成功')
  } catch (error: any) {
    if (error !== 'cancel') {
      // ElMessage.error('平仓失败: ' + error.message)
      console.error('平仓失败:', error)
    }
  }
}

const onLeverageChange = async (value: number) => {
  try {
    await tradingStore.setLeverage(props.symbol, value)
  } catch (error: any) {
    // ElMessage.error('设置杠杆失败: ' + error.message)
    console.error('设置杠杆失败:', error)
    // 恢复原值
    leverage.value = currentPosition.value?.leverage || 10
  }
}

// 订单确认弹窗
const showOrderConfirmation = (order: any): Promise<boolean> => {
  return new Promise((resolve) => {
    const orderValue = tradingStore.calculateOrderValue(
      order.price || props.currentPrice, 
      order.quantity
    )
    const commission = tradingStore.calculateCommission(orderValue)
    
    // 简化确认逻辑，避免ElMessageBox依赖
    const confirmed = confirm(`确认提交订单？
交易对: ${order.symbol}
方向: ${order.side === 'BUY' ? '买入' : '卖出'}
类型: ${getOrderTypeText(order.type)}
数量: ${order.quantity}
${order.price ? `价格: ${order.price}` : ''}
预估价值: ${formatBalance(orderValue)} ${quoteAsset.value}
预估手续费: ${formatBalance(commission)} ${quoteAsset.value}`)
    
    resolve(confirmed)
  })
}

// 工具函数
const formatBalance = (balance: number): string => {
  return balance.toFixed(2)
}

const formatAmount = (amount: number): string => {
  return amount.toFixed(quantityPrecision.value)
}

const formatPrice = (price: number): string => {
  return price.toFixed(pricePrecision.value)
}

const formatPnL = (pnl: number): string => {
  const sign = pnl >= 0 ? '+' : ''
  return `${sign}${pnl.toFixed(2)} ${quoteAsset.value}`
}

const getOrderTypeText = (type: string): string => {
  const typeMap = {
    'MARKET': '市价单',
    'LIMIT': '限价单',
    'STOP_LOSS': '止损单',
    'STOP_LOSS_LIMIT': '止损限价单',
    'TAKE_PROFIT': '止盈单',
    'TAKE_PROFIT_LIMIT': '止盈限价单'
  }
  return typeMap[type] || type
}

const playOrderSound = () => {
  // 播放订单提示音
  const audio = new Audio('/sounds/order-success.mp3')
  audio.play().catch(() => {
    // 忽略播放失败
  })
}

// 监听交易对变化
watch(() => props.symbol, () => {
  quickAmount.value = 0
  leverage.value = currentPosition.value?.leverage || 10
})
</script>

<style lang="scss" scoped>
.trading-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.trading-tabs {
  flex: 1;
  
  :deep(.el-tabs__content) {
    height: calc(100% - 40px);
    overflow-y: auto;
  }
}

.quick-actions {
  padding: 12px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  
  .quick-buttons {
    display: flex;
    gap: 8px;
    margin-bottom: 8px;
    
    .el-button {
      flex: 1;
      height: 40px;
      font-weight: 600;
      
      &.quick-buy {
        background: var(--success-color);
        border-color: var(--success-color);
        
        &:hover {
          background: var(--success-color-light);
        }
      }
      
      &.quick-sell {
        background: var(--error-color);
        border-color: var(--error-color);
        
        &:hover {
          background: var(--error-color-light);
        }
      }
    }
  }
  
  .quick-amount {
    .el-input-number {
      width: 100%;
    }
  }
}

.account-info {
  padding: 12px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  
  .balance-section {
    .balance-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
      
      &:last-child {
        margin-bottom: 0;
      }
      
      .label {
        font-size: 12px;
        color: var(--text-secondary);
      }
      
      .value {
        font-size: 12px;
        font-weight: 600;
        color: var(--text-primary);
      }
      
      .leverage-control {
        display: flex;
        align-items: center;
        gap: 8px;
        flex: 1;
        margin-left: 8px;
        
        .el-slider {
          flex: 1;
        }
        
        .leverage-value {
          font-size: 12px;
          font-weight: 600;
          color: var(--accent-primary);
          min-width: 30px;
        }
      }
    }
  }
}

.position-info {
  padding: 12px;
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-color);
  
  .position-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    
    h4 {
      font-size: 14px;
      font-weight: 600;
      color: var(--text-primary);
      margin: 0;
    }
  }
  
  .position-details {
    .position-item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 6px;
      
      &:last-child {
        margin-bottom: 0;
      }
      
      .label {
        font-size: 11px;
        color: var(--text-secondary);
      }
      
      .value {
        font-size: 11px;
        font-weight: 600;
        
        &.long {
          color: var(--success-color);
        }
        
        &.short {
          color: var(--error-color);
        }
        
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

.risk-warning {
  padding: 8px 12px;
  
  :deep(.el-alert) {
    .el-alert__title {
      font-size: 11px;
    }
  }
}

// 订单确认弹窗样式
:deep(.order-confirmation) {
  text-align: left;
  
  p {
    margin: 4px 0;
    font-size: 14px;
    
    strong {
      color: var(--text-primary);
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .quick-actions {
    .quick-buttons {
      .el-button {
        height: 36px;
        font-size: 12px;
      }
    }
  }
  
  .account-info,
  .position-info {
    padding: 8px;
    
    .balance-item,
    .position-item {
      .label,
      .value {
        font-size: 11px;
      }
    }
  }
}
</style>