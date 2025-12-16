<template>
  <div class="options-trading-form">
    <!-- VIP提示 -->
    <div class="vip-notice">
      <el-alert
        title="期权交易需要VIP权限"
        type="info"
        :closable="false"
        show-icon
      >
        <template #default>
          <p>期权交易功能仅对VIP用户开放</p>
          <el-button type="primary" size="small" @click="upgradeToVIP">
            升级VIP
          </el-button>
        </template>
      </el-alert>
    </div>

    <!-- 期权类型选择 -->
    <div class="option-type" v-if="isVIP">
      <el-radio-group v-model="optionType">
        <el-radio-button label="CALL">看涨期权</el-radio-button>
        <el-radio-button label="PUT">看跌期权</el-radio-button>
      </el-radio-group>
    </div>

    <!-- 到期时间 -->
    <div class="expiry-selection" v-if="isVIP">
      <div class="input-label">到期时间</div>
      <el-select v-model="selectedExpiry" placeholder="选择到期时间">
        <el-option 
          v-for="expiry in availableExpiries"
          :key="expiry.value"
          :label="expiry.label"
          :value="expiry.value"
        />
      </el-select>
    </div>

    <!-- 行权价格 -->
    <div class="strike-price" v-if="isVIP">
      <div class="input-label">
        <span>行权价格</span>
        <span class="current-price">当前: {{ formatPrice(currentPrice) }}</span>
      </div>
      <el-select v-model="strikePrice" placeholder="选择行权价格">
        <el-option 
          v-for="strike in availableStrikes"
          :key="strike"
          :label="formatPrice(strike)"
          :value="strike"
        />
      </el-select>
    </div>

    <!-- 期权信息 -->
    <div class="option-info" v-if="isVIP && selectedOption">
      <div class="info-item">
        <span class="label">期权价格</span>
        <span class="value">{{ formatPrice(selectedOption.premium) }}</span>
      </div>
      <div class="info-item">
        <span class="label">隐含波动率</span>
        <span class="value">{{ formatPercent(selectedOption.impliedVolatility) }}</span>
      </div>
      <div class="info-item">
        <span class="label">Delta</span>
        <span class="value">{{ selectedOption.delta.toFixed(4) }}</span>
      </div>
      <div class="info-item">
        <span class="label">Gamma</span>
        <span class="value">{{ selectedOption.gamma.toFixed(4) }}</span>
      </div>
      <div class="info-item">
        <span class="label">Theta</span>
        <span class="value">{{ selectedOption.theta.toFixed(4) }}</span>
      </div>
      <div class="info-item">
        <span class="label">Vega</span>
        <span class="value">{{ selectedOption.vega.toFixed(4) }}</span>
      </div>
    </div>

    <!-- 数量输入 -->
    <div class="quantity-input" v-if="isVIP">
      <div class="input-label">
        <span>合约数量</span>
        <span class="available">
          可用: {{ formatAmount(availableBalance) }} USDT
        </span>
      </div>
      <el-input-number
        v-model="quantity"
        :min="1"
        :max="maxQuantity"
        placeholder="输入合约数量"
      />
    </div>

    <!-- 订单预览 -->
    <div class="order-preview" v-if="isVIP && selectedOption">
      <div class="preview-item">
        <span class="label">期权费用</span>
        <span class="value">{{ formatAmount(totalPremium) }} USDT</span>
      </div>
      <div class="preview-item">
        <span class="label">手续费</span>
        <span class="value">{{ formatAmount(estimatedFee) }} USDT</span>
      </div>
      <div class="preview-item">
        <span class="label">总费用</span>
        <span class="value">{{ formatAmount(totalCost) }} USDT</span>
      </div>
    </div>

    <!-- 提交按钮 -->
    <div class="submit-section" v-if="isVIP">
      <el-button
        type="primary"
        size="large"
        :loading="isSubmitting"
        :disabled="!canSubmit"
        @click="submitOrder"
        class="submit-button"
      >
        {{ submitButtonText }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useUserStore } from '@/stores/user'
import { useTradingStore } from '@/stores/trading'

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
const userStore = useUserStore()
const tradingStore = useTradingStore()

// 响应式数据
const optionType = ref<'CALL' | 'PUT'>('CALL')
const selectedExpiry = ref('')
const strikePrice = ref(0)
const quantity = ref(1)
const isSubmitting = ref(false)

// 计算属性
const isVIP = computed(() => userStore.isVIP)

const availableExpiries = computed(() => {
  // 模拟可用的到期时间
  const now = new Date()
  const expiries = []
  
  // 添加周期权（每周五到期）
  for (let i = 1; i <= 4; i++) {
    const date = new Date(now)
    date.setDate(date.getDate() + (5 - date.getDay() + 7 * (i - 1)) % 7 + 7 * Math.floor((i - 1) / 1))
    expiries.push({
      label: `${date.getMonth() + 1}月${date.getDate()}日 (${i}周)`,
      value: date.getTime()
    })
  }
  
  // 添加月期权
  for (let i = 1; i <= 3; i++) {
    const date = new Date(now.getFullYear(), now.getMonth() + i, 0)
    expiries.push({
      label: `${date.getMonth() + 1}月${date.getDate()}日 (${i}月)`,
      value: date.getTime()
    })
  }
  
  return expiries
})

const availableStrikes = computed(() => {
  // 基于当前价格生成行权价格
  const strikes = []
  const basePrice = props.currentPrice
  const step = basePrice * 0.05 // 5%步长
  
  for (let i = -5; i <= 5; i++) {
    strikes.push(basePrice + (step * i))
  }
  
  return strikes.sort((a, b) => a - b)
})

const selectedOption = computed(() => {
  if (!selectedExpiry.value || !strikePrice.value) return null
  
  // 模拟期权数据
  const timeToExpiry = (selectedExpiry.value - Date.now()) / (1000 * 60 * 60 * 24 * 365)
  const moneyness = strikePrice.value / props.currentPrice
  
  return {
    premium: calculatePremium(optionType.value, moneyness, timeToExpiry),
    impliedVolatility: 0.25 + Math.random() * 0.3,
    delta: calculateDelta(optionType.value, moneyness),
    gamma: 0.01 + Math.random() * 0.02,
    theta: -0.05 - Math.random() * 0.1,
    vega: 0.1 + Math.random() * 0.2
  }
})

const availableBalance = computed(() => {
  return tradingStore.balance
})

const maxQuantity = computed(() => {
  if (!selectedOption.value) return 0
  return Math.floor(availableBalance.value / selectedOption.value.premium)
})

const totalPremium = computed(() => {
  if (!selectedOption.value) return 0
  return selectedOption.value.premium * quantity.value
})

const estimatedFee = computed(() => {
  return totalPremium.value * 0.001 // 0.1% 手续费
})

const totalCost = computed(() => {
  return totalPremium.value + estimatedFee.value
})

const canSubmit = computed(() => {
  return isVIP.value && 
         selectedExpiry.value && 
         strikePrice.value > 0 && 
         quantity.value > 0 && 
         totalCost.value <= availableBalance.value
})

const submitButtonText = computed(() => {
  if (isSubmitting.value) return '提交中...'
  return `买入 ${optionType.value === 'CALL' ? '看涨' : '看跌'}期权`
})

// 方法
const calculatePremium = (type: string, moneyness: number, timeToExpiry: number): number => {
  // 简化的Black-Scholes期权定价模型
  const volatility = 0.25
  const riskFreeRate = 0.05
  
  let intrinsicValue = 0
  if (type === 'CALL') {
    intrinsicValue = Math.max(props.currentPrice - strikePrice.value, 0)
  } else {
    intrinsicValue = Math.max(strikePrice.value - props.currentPrice, 0)
  }
  
  const timeValue = props.currentPrice * volatility * Math.sqrt(timeToExpiry) * 0.4
  
  return intrinsicValue + timeValue
}

const calculateDelta = (type: string, moneyness: number): number => {
  if (type === 'CALL') {
    return Math.max(0.1, Math.min(0.9, 0.5 + (1 - moneyness) * 0.5))
  } else {
    return Math.max(-0.9, Math.min(-0.1, -0.5 - (1 - moneyness) * 0.5))
  }
}

const submitOrder = async () => {
  try {
    isSubmitting.value = true
    
    const order = {
      symbol: props.symbol,
      type: 'OPTION',
      optionType: optionType.value,
      strikePrice: strikePrice.value,
      expiry: selectedExpiry.value,
      quantity: quantity.value,
      premium: selectedOption.value?.premium
    }
    
    emit('orderSubmit', order)
    
    // 重置表单
    quantity.value = 1
    
  } catch (error) {
    console.error('Submit option order error:', error)
  } finally {
    isSubmitting.value = false
  }
}

const upgradeToVIP = () => {
  // 跳转到VIP升级页面
  window.open('/vip', '_blank')
}

// 格式化函数
const formatPrice = (price: number): string => {
  return price.toFixed(2)
}

const formatAmount = (amount: number): string => {
  return amount.toFixed(2)
}

const formatPercent = (percent: number): string => {
  return (percent * 100).toFixed(2) + '%'
}
</script>

<style lang="scss" scoped>
.options-trading-form {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
}

.vip-notice {
  margin-bottom: 16px;
}

.option-type {
  margin-bottom: 16px;
  
  .el-radio-group {
    width: 100%;
    
    .el-radio-button {
      flex: 1;
    }
  }
}

.expiry-selection,
.strike-price,
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
    }
    
    .available {
      color: var(--text-tertiary);
    }
  }
  
  .el-select,
  .el-input-number {
    width: 100%;
  }
}

.option-info {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  
  .info-item {
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
      padding-top: 6px;
      border-top: 1px solid var(--border-color);
      font-weight: 600;
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
  .submit-button {
    width: 100%;
    height: 44px;
    font-size: 16px;
    font-weight: 600;
  }
}

// 响应式设计
@media (max-width: 768px) {
  .options-trading-form {
    padding: 12px;
  }
  
  .submit-section {
    .submit-button {
      height: 40px;
      font-size: 14px;
    }
  }
}
</style>