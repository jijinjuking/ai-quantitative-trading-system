<template>
  <div class="orders-list">
    <!-- 头部工具栏 -->
    <div class="orders-header">
      <div class="header-left">
        <h3>订单</h3>
        <el-badge :value="filteredOrders.length" type="primary" />
      </div>
      <div class="header-right">
        <el-button-group size="small">
          <el-button 
            :type="viewMode === 'open' ? 'primary' : 'default'"
            @click="viewMode = 'open'"
          >
            当前委托
          </el-button>
          <el-button 
            :type="viewMode === 'history' ? 'primary' : 'default'"
            @click="viewMode = 'history'"
          >
            历史订单
          </el-button>
        </el-button-group>
      </div>
    </div>

    <!-- 订单列表 -->
    <div class="orders-table">
      <el-table 
        :data="filteredOrders" 
        size="small"
        :show-header="true"
        height="100%"
      >
        <el-table-column prop="symbol" label="交易对" width="100" />
        <el-table-column prop="side" label="方向" width="60">
          <template #default="{ row }">
            <span :class="['side-tag', row.side]">
              {{ row.side === 'buy' ? '买' : '卖' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="type" label="类型" width="80">
          <template #default="{ row }">
            {{ getOrderTypeText(row.type) }}
          </template>
        </el-table-column>
        <el-table-column prop="quantity" label="数量" width="100">
          <template #default="{ row }">
            {{ formatQuantity(row.quantity) }}
          </template>
        </el-table-column>
        <el-table-column prop="price" label="价格" width="100">
          <template #default="{ row }">
            {{ row.type === 'market' ? '市价' : formatPrice(row.price) }}
          </template>
        </el-table-column>
        <el-table-column prop="filled" label="已成交" width="100">
          <template #default="{ row }">
            {{ formatQuantity(row.filled) }}
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="80">
          <template #default="{ row }">
            <span :class="['status-tag', row.status]">
              {{ getStatusText(row.status) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="time" label="时间" width="120">
          <template #default="{ row }">
            {{ formatTime(row.time) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="100" v-if="viewMode === 'open'">
          <template #default="{ row }">
            <el-button 
              size="mini" 
              type="danger" 
              @click="cancelOrder(row.id)"
              :disabled="row.status !== 'open'"
            >
              撤销
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 空状态 -->
    <div v-if="filteredOrders.length === 0" class="empty-state">
      <el-empty :description="viewMode === 'open' ? '暂无当前委托' : '暂无历史订单'" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

// Props
interface Props {
  symbol?: string
  filter?: 'open' | 'history'
}

const props = withDefaults(defineProps<Props>(), {
  symbol: '',
  filter: 'open'
})

// Emits
const emit = defineEmits<{
  orderCancel: [orderId: string]
}>()

// 响应式数据
const viewMode = ref<'open' | 'history'>(props.filter)

// 模拟订单数据
const orders = ref([
  {
    id: '1',
    symbol: 'BTCUSDT',
    side: 'buy',
    type: 'limit',
    quantity: 0.1,
    price: 49500,
    filled: 0,
    status: 'open',
    time: Date.now() - 300000
  },
  {
    id: '2',
    symbol: 'ETHUSDT',
    side: 'sell',
    type: 'market',
    quantity: 1.0,
    price: 0,
    filled: 1.0,
    status: 'filled',
    time: Date.now() - 600000
  },
  {
    id: '3',
    symbol: 'BTCUSDT',
    side: 'buy',
    type: 'stop',
    quantity: 0.05,
    price: 48000,
    filled: 0,
    status: 'cancelled',
    time: Date.now() - 900000
  }
])

// 计算属性
const filteredOrders = computed(() => {
  let filtered = orders.value

  // 按交易对筛选
  if (props.symbol) {
    filtered = filtered.filter(o => o.symbol === props.symbol)
  }

  // 按状态筛选
  if (viewMode.value === 'open') {
    filtered = filtered.filter(o => o.status === 'open' || o.status === 'partial')
  } else {
    filtered = filtered.filter(o => o.status === 'filled' || o.status === 'cancelled')
  }

  return filtered.sort((a, b) => b.time - a.time)
})

// 方法
const formatQuantity = (quantity: number) => {
  return quantity.toFixed(4)
}

const formatPrice = (price: number) => {
  return price.toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

const formatTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleTimeString()
}

const getOrderTypeText = (type: string) => {
  const types = {
    market: '市价',
    limit: '限价',
    stop: '止损',
    'stop-limit': '止损限价'
  }
  return types[type] || type
}

const getStatusText = (status: string) => {
  const statuses = {
    open: '待成交',
    partial: '部分成交',
    filled: '已成交',
    cancelled: '已撤销'
  }
  return statuses[status] || status
}

const cancelOrder = (orderId: string) => {
  emit('orderCancel', orderId)
}
</script>

<style lang="scss" scoped>
.orders-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.orders-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;

    h3 {
      margin: 0;
      font-size: 14px;
      font-weight: 600;
      color: var(--text-primary);
    }
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
}

.orders-table {
  flex: 1;
  overflow: hidden;

  :deep(.el-table) {
    background: var(--bg-primary);
    color: var(--text-primary);

    .el-table__header {
      background: var(--bg-secondary);

      th {
        background: var(--bg-secondary);
        color: var(--text-secondary);
        border-bottom: 1px solid var(--border-color);
        font-weight: 500;
        font-size: 12px;
      }
    }

    .el-table__body {
      tr {
        background: var(--bg-primary);

        &:hover {
          background: var(--bg-tertiary);
        }

        td {
          border-bottom: 1px solid var(--border-color);
          color: var(--text-primary);
          font-size: 12px;
        }
      }
    }
  }
}

.side-tag {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;

  &.buy {
    color: var(--success-color);
    background: var(--success-bg);
  }

  &.sell {
    color: var(--error-color);
    background: var(--error-bg);
  }
}

.status-tag {
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;

  &.open {
    color: var(--warning-color);
    background: var(--warning-bg);
  }

  &.partial {
    color: var(--info-color);
    background: var(--info-bg);
  }

  &.filled {
    color: var(--success-color);
    background: var(--success-bg);
  }

  &.cancelled {
    color: var(--text-tertiary);
    background: var(--bg-tertiary);
  }
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}
</style>