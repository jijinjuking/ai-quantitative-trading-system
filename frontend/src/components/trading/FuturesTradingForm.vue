<template>
  <div class="futures-trading-form">
    <!-- 买卖切换 -->
    <div class="side-selector">
      <el-button-group>
        <el-button 
          :type="side === 'BUY' ? 'success' : 'default'"
          @click="side = 'BUY'"
          class="buy-button"
        >
          做多
        </el-button>
        <el-button 
          :type="side === 'SELL' ? 'danger' : 'default'"
          @click="side = 'SELL'"
          class="sell-button"
        >
          做空
        </el-button>
      </el-button-group>
    </div>

    <!-- 杠杆设置 -->
    <div class="leverage-section">
      <div class="input-label">
        <span>杠杆倍数</span>
        <span class="leverage-value">{{ leverage }}x</span>
      </div>
      <el-slider
        v-model="leverage"
        :min="1"
        :max="maxLeverage"
        :step="1"
        @change="onLeverageChange"
        class="leverage-slider"
      />
      <div class="leverage-presets">
        <el-button 
          v-for="preset in leveragePresets"
          :key="preset"
          size="small"
          :type="leverage === preset ? 'primary' : 'default'"
          @click="setLeverage(preset)"
        >
          {{ preset }}x
        </el-button>
      </div>
    </div>

    <!-- 订单类型 -->
    <div class="order-type">
      <el-select v-model="orderType" placeholder="订单类型">
        <el-option label="市价单" value="MARKET" />
        <el-option label="限价单" value="LIMIT" />
        <el-option label="止损市价单" value="STOP_MARKET" />
        <el-option label="止损限价单" value="STOP_LIMIT" />
        <el-option label="止盈市价单" value="TAKE_PROFIT_MARKET" />
        <el-option label="止盈限价单" value="TAKE_PROFIT_LIMIT" />
      </el-select>
    </div>

    <!-- 价格输入 -->
    <div class="price-input" v-if="needsPriceInput">
      <div class="input-label">
        <span>价格</span>
        <span class="current-price" @click="useCurrentPrice">
          {{ formatPrice(currentPrice) }}
        </span>
      </div>
      <el-input-number
        v-model="price"
        :precision="pricePrecision"
        :step="priceStep"
        :min="0"
        placeholder="输入价格"
        class="price-number-input"
      />
    </div>

    <!-- 触发价格 -->
    <div class="trigger-price-input" v-if="needsTriggerPrice">
      <div class="input-label">
        <span>触发价格</span>
      </div>
      <el-input-number
        v-model="triggerPrice"
        :precision="pricePrecision"
        :step="priceStep"
        :min="0"
        placeholder="输入触发价格"
      />
    </div>

    <!-- 数量输入方式 -->
    <div class="quantity-mode">
      <el-radio-group v-model="quantityMode">
        <el-radio-button label="quantity">数量</el-radio-button>
        <el-radio-button label="value">价值</el-radio-button>
      </el-radio-group>
    </div>

    <!-- 数量输入 -->
    <div class="quantity-input" v-if="quantityMode === 'quantity'">
      <div class="input-label">
        <span>数量</span>
        <span class="available">
          可开仓: {{ formatAmount(maxQuantity) }} {{ baseAsset }}
        </span>
      </div>
      <el-input-number
        v-model="quantity"
        :precision="quantityPrecision"
        :step="quantityStep"
        :min="0"
        :max="maxQuantity"
        placeholder="输入数量"
        class="quantity-number-input"
      />
      <div class="quantity-buttons">
        <el-button size="small" @click="setQuantityPercent(25)">25%</el-button>
        <el-button size="small" @click="setQuantityPercent(50)">50%</el-button>
        <el-button size="small" @click="setQuantityPercent(75)">75%</el-button>
        <el-button size="small" @click="setQuantityPercent(100)">100%</el-button>
      </div>
    </div>

    <!-- 价值输入 -->
    <div class="value-input" v-else>
      <div class="input-label">
        <span>价值 ({{ quoteAsset }})</span>
        <span class="available">
          可用: {{ formatAmount(availableBalance) }} {{ quoteAsset }}
        </span>
      </div>
      <el-input-number
        v-model="orderValue"
        :precision="2"
        :min="0"
        :max="maxOrderValue"
        placeholder="输入价值"
        class="value-number-input"
      />
      <div class="value-buttons">
        <el-button size="small" @click="setValuePercent(25)">25%</el-button>
        <el-button size="small" @click="setValuePercent(50)">50%</el-button>
        <el-button size="small" @click="setValuePercent(75)">75%</el-button>
        <el-button size="small" @click="setValuePercent(100)">100%</el-button>
      </div>
    </div>

    <!-- 止盈止损 -->
    <div class="tp-sl-section">
      <el-collapse>
        <el-collapse-item title="止盈止损" name="tpsl">
          <!-- 止盈 -->
          <div class="tp-section">
            <el-checkbox v-model="enableTakeProfit">止盈</el-checkbox>
            <div v-if="enableTakeProfit" class="tp-inputs">
              <el-input-number
                v-model="takeProfitPrice"
                :precision="pricePrecision"
                placeholder="止盈价格"
                size="small"
              />
              <el-select v-model="takeProfitType" size="small">
                <el-option label="市价" value="MARKET" />
                <el-option label="限价" value="LIMIT" />
              </el-select>
            </div>
          </div>

          <!-- 止损 -->
          <div class="sl-section">
            <el-checkbox v-model="enableStopLoss">止损</el-checkbox>
            <div v-if="enableStopLoss" class="sl-inputs">
              <el-input-number
                v-model="stopLossPrice"
                :precision="pricePrecision"
                placeholder="止损价格"
                size="small"
              />
              <el-select v-model="stopLossType" size="small">
                <el-option label="市价" value="MARKET" />
                <el-option label="限价" value="LIMIT" />
              </el-select>
            </div>
          </div>
        </el-collapse-item>
      </el-collapse>
    </div>

    <!-- 高级选项 -->
    <div class="advanced-options">
      <el-collapse>
        <el-collapse-item title="高级选项" name="advanced">
          <!-- 时效性 -->
          <div class="time-in-force">
            <div class="input-label">时效性</div>
            <el-select v-model="timeInForce">
              <el-option label="GTC (撤销前有效)" value="GTC" />
              <el-option label="IOC (立即成交或取消)" value="IOC" />
              <el-option label="FOK (全部成交或取消)" value="FOK" />
              <el-option label="GTX (被动委托)" value="GTX" />
            </el-select>
          </div>

          <!-- 只减仓 -->
          <div class="reduce-only">
            <el-checkbox v-model="reduceOnly">只减仓</el-checkbox>
          </div>

          <!-- 平仓单 -->
          <div class="close-position">
            <el-checkbox v-model="closePosition">平仓单</el-checkbox>
          </div>
        </el-collapse-item>
      </el-collapse>
    </div>

    <!-- 订单预览 -->
    <div class="order-preview">
      <div class="preview-item">
        <span class="label">开仓价格</span>
        <span class="value">{{ formatPrice(effectivePrice) }}</span>
      </div>
      <div class="preview-item">
        <span class="label">开仓数量</span>
        <span class="value">{{ formatAmount(effectiveQuantity) }} {{ baseAsset }}</span>
      </div>
      <div class="preview-item">
        <span class="label">名义价值</span>
        <span class="value">{{ formatAmount(notionalValue) }} {{ quoteAsset }}</span>
      </div>
      <div class="preview-item">
        <span class="label">初始保证金</span>
        <span class="value">{{ formatAmount(initialMargin) }} {{ quoteAsset }}</span>
      </div>
      <div class="preview-item">
        <span class="label">预估手续费</span>
        <span class="value">{{ formatAmount(estimatedFee) }} {{ quoteAsset }}</span>
      </div>
      <div class="preview-item" v-if="enableTakeProfit">
        <span class="label">止盈价格</span>
        <span class="value positive">{{ formatPrice(takeProfitPrice) }}</span>
      </div>
      <div class="preview-item" v-if="enableStopLoss">
        <span class="label">止损价格</span>
        <span class="value negative">{{ formatPrice(stopLossPrice) }}</span>
      </div>
    </div>

    <!-- 风险指标 -->
    <div class="risk-indicators">
      <div class="risk-item">
        <span class="label">强平价格</span>
        <span class="value negative">{{ formatPrice(liquidationPrice) }}</span>
      </div>
      <div class="risk-item">
        <span class="label">保证金率</span>
        <span class="value" :class="marginRatioClass">{{ formatPercent(marginRatio) }}</span>
      </div>
    </div>

    <!-- 提交按钮 -->
    <div class="submit-section">
      <el-button
        :type="side === 'BUY' ? 'success' : 'danger'"
        size="large"
        :loading="isSubmitting"
        :disabled="!canSubmit"
        @click="submitOrder"
        class="submit-button"
      >
        {{ submitButtonText }}
      </el-button>
    </div>

    <!-- 风险提示 -->
    <div class="risk-warning" v-if="showRiskWarning">
      <el-alert
        :title="riskWarningText"
        type="error"
        :closable="false"
        show-icon
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTradingStore } from '@/stores/trading'
import { useMarketStore } from '@/stores/market'
import { useUserStore } from '@/stores/user'

// Props
interface Props {
  symbol: string
  currentPrice: number
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  orderSubmit: [order: any]
}>()

// 状态管理
const tradingStore = useTradingStore()
const marketStore = useMarketStore()
const userStore = useUserStore()

// 响应式数据
const side = ref<'BUY' | 'SELL'>('BUY')
const orderType = ref('LIMIT')
const leverage = ref(10)
const price = ref(0)
const triggerPrice = ref(0)
const quantityMode = ref('quantity')
const quantity = ref(0)
const orderValue = ref(0)
const timeInForce = ref('GTC')
const reduceOnly = ref(false)
const closePosition = ref(false)
const isSubmitting = ref(false)

// 止盈止损
const enableTakeProfit = ref(false)
const takeProfitPrice = ref(0)
const takeProfitType = ref('MARKET')
const enableStopLoss = ref(false)
const stopLossPrice = ref(0)
const stopLossType = ref('MARKET')

// 计算属性
const baseAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[0] || props.symbol.substring(0, 3)
})

const quoteAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[1] || props.symbol.substring(3)
})

const maxLeverage = computed(() => {
  return userStore.user?.kyc.limits.maxLeverage || 20
})

const leveragePresets = computed(() => {
  const presets = [1, 2, 3, 5, 10, 20, 50, 100]
  return presets.filter(p => p <= maxLeverage.value)
})

const pricePrecision = computed(() => {
  return marketStore.getPricePrecision(props.symbol)
})

const quantityPrecision = computed(() => {
  return marketStore.getQuantityPrecision(props.symbol)
})

const priceStep = computed(() => {
  return Math.pow(10, -pricePrecision.value)
})

const quantityStep = computed(() => {
  return Math.pow(10, -quantityPrecision.value)
})

const availableBalance = computed(() => {
  return tradingStore.marginInfo?.availableMargin || 0
})

const maxQuantity = computed(() => {
  const orderPrice = effectivePrice.value
  if (orderPrice <= 0) return 0
  
  const maxNotional = availableBalance.value * leverage.value
  return maxNotional / orderPrice
})

const maxOrderValue = computed(() => {
  return availableBalance.value * leverage.value
})

const effectivePrice = computed(() => {
  return orderType.value === 'MARKET' ? props.currentPrice : price.value
})

const effectiveQuantity = computed(() => {
  if (quantityMode.value === 'quantity') {
    return quantity.value
  } else {
    return orderValue.value / effectivePrice.value
  }
})

const notionalValue = computed(() => {
  return effectivePrice.value * effectiveQuantity.value
})

const initialMargin = computed(() => {
  return notionalValue.value / leverage.value
})

const estimatedFee = computed(() => {
  return tradingStore.calculateCommission(notionalValue.value, 0.0004) // 合约手续费率
})

const liquidationPrice = computed(() => {
  if (effectiveQuantity.value === 0) return 0
  
  const maintenanceMargin = notionalValue.value * 0.005 // 维持保证金率 0.5%
  const entryPrice = effectivePrice.value
  
  if (side.value === 'BUY') {
    return entryPrice * (1 - (initialMargin.value - maintenanceMargin) / notionalValue.value)
  } else {
    return entryPrice * (1 + (initialMargin.value - maintenanceMargin) / notionalValue.value)
  }
})

const marginRatio = computed(() => {
  if (availableBalance.value === 0) return 0
  return (initialMargin.value / availableBalance.value) * 100
})

const marginRatioClass = computed(() => {
  if (marginRatio.value > 80) return 'negative'
  if (marginRatio.value > 60) return 'warning'
  return 'positive'
})

const needsPriceInput = computed(() => {
  return ['LIMIT', 'STOP_LIMIT', 'TAKE_PROFIT_LIMIT'].includes(orderType.value)
})

const needsTriggerPrice = computed(() => {
  return orderType.value.includes('STOP') || orderType.value.includes('TAKE_PROFIT')
})

const canSubmit = computed(() => {
  if (effectiveQuantity.value <= 0) return false
  if (needsPriceInput.value && price.value <= 0) return false
  if (needsTriggerPrice.value && triggerPrice.value <= 0) return false
  if (initialMargin.value > availableBalance.value) return false
  return true
})

const submitButtonText = computed(() => {
  if (isSubmitting.value) return '提交中...'
  
  const action = side.value === 'BUY' ? '做多' : '做空'
  const type = orderType.value === 'MARKET' ? '市价' : '限价'
  
  return `${type}${action} ${baseAsset.value}`
})

const showRiskWarning = computed(() => {
  return leverage.value > 10 || marginRatio.value > 70
})

const riskWarningText = computed(() => {
  if (leverage.value > 50) {
    return '极高杠杆风险：可能导致快速爆仓，请谨慎操作！'
  } else if (leverage.value > 20) {
    return '高杠杆风险：请注意风险控制，建议设置止损'
  } else if (marginRatio.value > 70) {
    return '保证金使用率过高，请注意风险控制'
  }
  return ''
})

// 初始化
watch(() => props.currentPrice, (newPrice) => {
  if (price.value === 0 || orderType.value === 'MARKET') {
    price.value = newPrice
  }
  
  // 自动设置止盈止损价格
  if (enableTakeProfit.value && takeProfitPrice.value === 0) {
    takeProfitPrice.value = side.value === 'BUY' ? newPrice * 1.02 : newPrice * 0.98
  }
  
  if (enableStopLoss.value && stopLossPrice.value === 0) {
    stopLossPrice.value = side.value === 'BUY' ? newPrice * 0.98 : newPrice * 1.02
  }
}, { immediate: true })

// 方法
const useCurrentPrice = () => {
  price.value = props.currentPrice
}

const setLeverage = (value: number) => {
  leverage.value = value
}

const onLeverageChange = async (value: number) => {
  try {
    await tradingStore.setLeverage(props.symbol, value)
  } catch (error) {
    ElMessage.error('设置杠杆失败: ' + error.message)
  }
}

const setQuantityPercent = (percent: number) => {
  quantity.value = (maxQuantity.value * percent) / 100
}

const setValuePercent = (percent: number) => {
  orderValue.value = (maxOrderValue.value * percent) / 100
}

const submitOrder = async () => {
  try {
    isSubmitting.value = true

    const order = {
      symbol: props.symbol,
      side: side.value,
      type: orderType.value,
      quantity: effectiveQuantity.value,
      ...(needsPriceInput.value && { price: price.value }),
      ...(needsTriggerPrice.value && { stopPrice: triggerPrice.value }),
      timeInForce: timeInForce.value,
      ...(reduceOnly.value && { reduceOnly: true }),
      ...(closePosition.value && { closePosition: true })
    }

    emit('orderSubmit', order)
    
    // 提交止盈止损订单
    if (enableTakeProfit.value) {
      await submitTPSLOrder('TAKE_PROFIT', takeProfitPrice.value, takeProfitType.value)
    }
    
    if (enableStopLoss.value) {
      await submitTPSLOrder('STOP_LOSS', stopLossPrice.value, stopLossType.value)
    }
    
    // 重置表单
    resetForm()
    
  } catch (error) {
    console.error('Submit order error:', error)
  } finally {
    isSubmitting.value = false
  }
}

const submitTPSLOrder = async (type: string, triggerPrice: number, orderType: string) => {
  const tpslOrder = {
    symbol: props.symbol,
    side: side.value === 'BUY' ? 'SELL' : 'BUY', // 反向平仓
    type: orderType === 'MARKET' ? `${type}_MARKET` : `${type}_LIMIT`,
    quantity: effectiveQuantity.value,
    stopPrice: triggerPrice,
    ...(orderType === 'LIMIT' && { price: triggerPrice }),
    timeInForce: 'GTC',
    reduceOnly: true
  }
  
  await tradingStore.submitOrder(tpslOrder)
}

const resetForm = () => {
  quantity.value = 0
  orderValue.value = 0
  if (orderType.value !== 'MARKET') {
    price.value = props.currentPrice
  }
  triggerPrice.value = 0
  enableTakeProfit.value = false
  takeProfitPrice.value = 0
  enableStopLoss.value = false
  stopLossPrice.value = 0
  reduceOnly.value = false
  closePosition.value = false
}

// 格式化函数
const formatPrice = (price: number) => {
  return price.toFixed(pricePrecision.value)
}

const formatAmount = (amount: number) => {
  return amount.toFixed(quantityPrecision.value)
}

const formatPercent = (percent: number) => {
  return percent.toFixed(2) + '%'
}

// 监听数量模式变化
watch(quantityMode, (newMode) => {
  if (newMode === 'quantity') {
    orderValue.value = 0
  } else {
    quantity.value = 0
  }
})

// 监听止盈止损开关
watch(enableTakeProfit, (enabled) => {
  if (enabled && takeProfitPrice.value === 0) {
    takeProfitPrice.value = side.value === 'BUY' ? props.currentPrice * 1.02 : props.currentPrice * 0.98
  }
})

watch(enableStopLoss, (enabled) => {
  if (enabled && stopLossPrice.value === 0) {
    stopLossPrice.value = side.value === 'BUY' ? props.currentPrice * 0.98 : props.currentPrice * 1.02
  }
})
</script>

<style lang="scss" scoped>
.futures-trading-form {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
}

.side-selector {
  margin-bottom: 16px;
  
  .el-button-group {
    width: 100%;
    
    .el-button {
      flex: 1;
    }
  }
}

.leverage-section {
  margin-bottom: 16px;
  
  .input-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    font-size: 12px;
    color: var(--text-secondary);
    
    .leverage-value {
      font-weight: 600;
      color: var(--accent-primary);
    }
  }
  
  .leverage-slider {
    margin-bottom: 8px;
  }
  
  .leverage-presets {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
    
    .el-button {
      flex: 1;
      min-width: 40px;
      font-size: 11px;
    }
  }
}

.quantity-mode {
  margin-bottom: 12px;
  
  .el-radio-group {
    width: 100%;
    
    .el-radio-button {
      flex: 1;
    }
  }
}

.order-preview {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  
  .preview-item {
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
      color: var(--text-primary);
      
      &.positive {
        color: var(--success-color);
      }
      
      &.negative {
        color: var(--error-color);
      }
    }
  }
}

.risk-indicators {
  margin-bottom: 16px;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  border-left: 3px solid var(--warning-color);
  
  .risk-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
    
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
      
      &.positive {
        color: var(--success-color);
      }
      
      &.warning {
        color: var(--warning-color);
      }
      
      &.negative {
        color: var(--error-color);
      }
    }
  }
}

.tp-sl-section {
  margin-bottom: 16px;
  
  .tp-section,
  .sl-section {
    margin-bottom: 12px;
    
    .tp-inputs,
    .sl-inputs {
      display: flex;
      gap: 8px;
      margin-top: 8px;
      
      .el-input-number {
        flex: 2;
      }
      
      .el-select {
        flex: 1;
      }
    }
  }
}

.submit-section {
  margin-bottom: 16px;
  
  .submit-button {
    width: 100%;
    height: 44px;
    font-size: 16px;
    font-weight: 600;
  }
}

.risk-warning {
  :deep(.el-alert) {
    .el-alert__title {
      font-size: 11px;
    }
  }
}

// 复用样式
.price-input,
.trigger-price-input,
.quantity-input,
.value-input {
  margin-bottom: 16px;
  
  .input-label {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    font-size: 12px;
    color: var(--text-secondary);
    
    .current-price {
      color: var(--accent-primary);
      cursor: pointer;
      
      &:hover {
        text-decoration: underline;
      }
    }
    
    .available {
      color: var(--text-tertiary);
    }
  }
  
  .price-number-input,
  .quantity-number-input,
  .value-number-input {
    width: 100%;
    margin-bottom: 8px;
  }
  
  .quantity-buttons,
  .value-buttons {
    display: flex;
    gap: 4px;
    
    .el-button {
      flex: 1;
      font-size: 11px;
    }
  }
}

.advanced-options {
  margin-bottom: 16px;
  
  .time-in-force {
    margin-bottom: 12px;
    
    .input-label {
      margin-bottom: 4px;
      font-size: 12px;
      color: var(--text-secondary);
    }
    
    .el-select {
      width: 100%;
    }
  }
  
  .reduce-only,
  .close-position {
    margin-bottom: 8px;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .futures-trading-form {
    padding: 12px;
  }
  
  .leverage-presets {
    .el-button {
      font-size: 10px;
      padding: 4px 6px;
    }
  }
  
  .submit-section {
    .submit-button {
      height: 40px;
      font-size: 14px;
    }
  }
}
</style>