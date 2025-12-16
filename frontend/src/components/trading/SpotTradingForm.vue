<template>
  <div class="spot-trading-form">
    <!-- 买卖切换 -->
    <div class="side-selector">
      <el-button-group>
        <el-button 
          :type="side === 'BUY' ? 'success' : 'default'"
          @click="side = 'BUY'"
          class="buy-button"
        >
          买入
        </el-button>
        <el-button 
          :type="side === 'SELL' ? 'danger' : 'default'"
          @click="side = 'SELL'"
          class="sell-button"
        >
          卖出
        </el-button>
      </el-button-group>
    </div>

    <!-- 订单类型 -->
    <div class="order-type">
      <el-select v-model="orderType" placeholder="订单类型">
        <el-option label="市价单" value="MARKET" />
        <el-option label="限价单" value="LIMIT" />
        <el-option label="止损单" value="STOP_LOSS" />
        <el-option label="止盈单" value="TAKE_PROFIT" />
      </el-select>
    </div>

    <!-- 价格输入 -->
    <div class="price-input" v-if="orderType === 'LIMIT'">
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
      <div class="price-buttons">
        <el-button size="small" @click="adjustPrice(-5)">-5%</el-button>
        <el-button size="small" @click="adjustPrice(-1)">-1%</el-button>
        <el-button size="small" @click="adjustPrice(1)">+1%</el-button>
        <el-button size="small" @click="adjustPrice(5)">+5%</el-button>
      </div>
    </div>

    <!-- 止损价格 -->
    <div class="stop-price-input" v-if="orderType.includes('STOP')">
      <div class="input-label">
        <span>{{ orderType === 'STOP_LOSS' ? '止损价格' : '止盈价格' }}</span>
      </div>
      <el-input-number
        v-model="stopPrice"
        :precision="pricePrecision"
        :step="priceStep"
        :min="0"
        placeholder="输入触发价格"
      />
    </div>

    <!-- 数量输入 -->
    <div class="quantity-input">
      <div class="input-label">
        <span>数量</span>
        <span class="available">
          可用: {{ formatAmount(availableAmount) }} {{ side === 'BUY' ? quoteAsset : baseAsset }}
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
            </el-select>
          </div>

          <!-- 冰山订单 -->
          <div class="iceberg-order">
            <el-checkbox v-model="icebergOrder">冰山订单</el-checkbox>
            <el-input-number
              v-if="icebergOrder"
              v-model="icebergQty"
              :precision="quantityPrecision"
              :min="0"
              placeholder="显示数量"
              size="small"
            />
          </div>

          <!-- 只减仓 -->
          <div class="reduce-only" v-if="side === 'SELL'">
            <el-checkbox v-model="reduceOnly">只减仓</el-checkbox>
          </div>
        </el-collapse-item>
      </el-collapse>
    </div>

    <!-- 订单预览 -->
    <div class="order-preview">
      <div class="preview-item">
        <span class="label">预估成交额</span>
        <span class="value">{{ formatAmount(estimatedValue) }} {{ quoteAsset }}</span>
      </div>
      <div class="preview-item">
        <span class="label">预估手续费</span>
        <span class="value">{{ formatAmount(estimatedFee) }} {{ quoteAsset }}</span>
      </div>
      <div class="preview-item" v-if="side === 'BUY'">
        <span class="label">预计获得</span>
        <span class="value">{{ formatAmount(estimatedReceive) }} {{ baseAsset }}</span>
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
    <div class="risk-tips" v-if="showRiskTips">
      <el-alert
        :title="riskTipsText"
        type="warning"
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

// 响应式数据
const side = ref<'BUY' | 'SELL'>('BUY')
const orderType = ref('LIMIT')
const price = ref(0)
const stopPrice = ref(0)
const quantity = ref(0)
const timeInForce = ref('GTC')
const icebergOrder = ref(false)
const icebergQty = ref(0)
const reduceOnly = ref(false)
const isSubmitting = ref(false)

// 计算属性
const baseAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[0] || props.symbol.substring(0, 3)
})

const quoteAsset = computed(() => {
  const parts = props.symbol.split('/')
  return parts[1] || props.symbol.substring(3)
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

const availableAmount = computed(() => {
  if (side.value === 'BUY') {
    const balance = tradingStore.assets.find(a => a.asset === quoteAsset.value)
    return balance?.free || 0
  } else {
    const balance = tradingStore.assets.find(a => a.asset === baseAsset.value)
    return balance?.free || 0
  }
})

const maxQuantity = computed(() => {
  if (side.value === 'BUY') {
    const orderPrice = orderType.value === 'MARKET' ? props.currentPrice : price.value
    return orderPrice > 0 ? availableAmount.value / orderPrice : 0
  } else {
    return availableAmount.value
  }
})

const estimatedValue = computed(() => {
  const orderPrice = orderType.value === 'MARKET' ? props.currentPrice : price.value
  return orderPrice * quantity.value
})

const estimatedFee = computed(() => {
  return tradingStore.calculateCommission(estimatedValue.value)
})

const estimatedReceive = computed(() => {
  if (side.value === 'BUY') {
    return quantity.value - (estimatedFee.value / (orderType.value === 'MARKET' ? props.currentPrice : price.value))
  } else {
    return estimatedValue.value - estimatedFee.value
  }
})

const canSubmit = computed(() => {
  if (quantity.value <= 0) return false
  if (orderType.value === 'LIMIT' && price.value <= 0) return false
  if (orderType.value.includes('STOP') && stopPrice.value <= 0) return false
  if (estimatedValue.value > availableAmount.value && side.value === 'BUY') return false
  return true
})

const submitButtonText = computed(() => {
  if (isSubmitting.value) return '提交中...'
  
  const action = side.value === 'BUY' ? '买入' : '卖出'
  const type = orderType.value === 'MARKET' ? '市价' : '限价'
  
  return `${type}${action} ${baseAsset.value}`
})

const showRiskTips = computed(() => {
  return orderType.value === 'MARKET' && estimatedValue.value > 1000
})

const riskTipsText = computed(() => {
  return '市价单将立即执行，请确认价格和数量无误'
})

// 初始化
watch(() => props.currentPrice, (newPrice) => {
  if (price.value === 0 || orderType.value === 'MARKET') {
    price.value = newPrice
  }
}, { immediate: true })

// 方法
const useCurrentPrice = () => {
  price.value = props.currentPrice
}

const adjustPrice = (percent: number) => {
  const adjustment = props.currentPrice * (percent / 100)
  price.value = Math.max(0, props.currentPrice + adjustment)
}

const setQuantityPercent = (percent: number) => {
  quantity.value = (maxQuantity.value * percent) / 100
}

const submitOrder = async () => {
  try {
    isSubmitting.value = true

    const order = {
      symbol: props.symbol,
      side: side.value,
      type: orderType.value,
      quantity: quantity.value,
      ...(orderType.value === 'LIMIT' && { price: price.value }),
      ...(orderType.value.includes('STOP') && { stopPrice: stopPrice.value }),
      timeInForce: timeInForce.value,
      ...(icebergOrder.value && { icebergQty: icebergQty.value }),
      ...(reduceOnly.value && { reduceOnly: true })
    }

    emit('orderSubmit', order)
    
    // 重置表单
    resetForm()
    
  } catch (error) {
    console.error('Submit order error:', error)
  } finally {
    isSubmitting.value = false
  }
}

const resetForm = () => {
  quantity.value = 0
  if (orderType.value === 'LIMIT') {
    price.value = props.currentPrice
  }
  stopPrice.value = 0
  icebergOrder.value = false
  icebergQty.value = 0
  reduceOnly.value = false
}

// 格式化函数
const formatPrice = (price: number) => {
  return price.toFixed(pricePrecision.value)
}

const formatAmount = (amount: number) => {
  return amount.toFixed(quantityPrecision.value)
}

// 监听订单类型变化
watch(orderType, (newType) => {
  if (newType === 'MARKET') {
    price.value = props.currentPrice
  }
})
</script>

<style lang="scss" scoped>
.spot-trading-form {
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
      
      &.buy-button {
        &.el-button--success {
          background: var(--success-color);
          border-color: var(--success-color);
        }
      }
      
      &.sell-button {
        &.el-button--danger {
          background: var(--error-color);
          border-color: var(--error-color);
        }
      }
    }
  }
}

.order-type {
  margin-bottom: 16px;
  
  .el-select {
    width: 100%;
  }
}

.price-input,
.stop-price-input,
.quantity-input {
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
  .quantity-number-input {
    width: 100%;
    margin-bottom: 8px;
  }
  
  .price-buttons,
  .quantity-buttons {
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
  
  .iceberg-order,
  .reduce-only {
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    
    .el-input-number {
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
      font-size: 12px;
      color: var(--text-secondary);
    }
    
    .value {
      font-size: 12px;
      font-weight: 600;
      color: var(--text-primary);
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

.risk-tips {
  :deep(.el-alert) {
    .el-alert__title {
      font-size: 11px;
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .spot-trading-form {
    padding: 12px;
  }
  
  .price-buttons,
  .quantity-buttons {
    .el-button {
      font-size: 10px;
      padding: 4px 8px;
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