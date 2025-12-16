<template>
  <div class="positions-list">
    <!-- 头部工具栏 -->
    <div class="positions-header">
      <div class="header-left">
        <h3>持仓</h3>
        <el-badge :value="filteredPositions.length" type="primary" />
      </div>
      <div class="header-right">
        <el-button-group size="small">
          <el-button 
            :type="viewMode === 'all' ? 'primary' : 'default'"
            @click="viewMode = 'all'"
          >
            全部
          </el-button>
          <el-button 
            :type="viewMode === 'long' ? 'primary' : 'default'"
            @click="viewMode = 'long'"
          >
            多头
          </el-button>
          <el-button 
            :type="viewMode === 'short' ? 'primary' : 'default'"
            @click="viewMode = 'short'"
          >
            空头
          </el-button>
        </el-button-group>
      </div>
    </div>

    <!-- 持仓列表 -->
    <div class="positions-table">
      <el-table 
        :data="filteredPositions" 
        size="small"
        :show-header="true"
        height="100%"
      >
        <el-table-column prop="symbol" label="交易对" width="100" />
        <el-table-column prop="side" label="方向" width="60">
          <template #default="{ row }">
            <span :class="['side-tag', row.side]">
              {{ row.side === 'long' ? '多' : '空' }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="size" label="数量" width="100">
          <template #default="{ row }">
            {{ formatQuantity(row.size) }}
          </template>
        </el-table-column>
        <el-table-column prop="entryPrice" label="开仓价" width="100">
          <template #default="{ row }">
            {{ formatPrice(row.entryPrice) }}
          </template>
        </el-table-column>
        <el-table-column prop="markPrice" label="标记价" width="100">
          <template #default="{ row }">
            {{ formatPrice(row.markPrice) }}
          </template>
        </el-table-column>
        <el-table-column prop="pnl" label="盈亏" width="100">
          <template #default="{ row }">
            <span :class="['pnl-value', row.pnl >= 0 ? 'positive' : 'negative']">
              {{ formatPnL(row.pnl) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button 
              size="mini" 
              type="danger" 
              @click="closePosition(row.id)"
            >
              平仓
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <!-- 空状态 -->
    <div v-if="filteredPositions.length === 0" class="empty-state">
      <el-empty description="暂无持仓" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

// Props
interface Props {
  symbol?: string
}

const props = withDefaults(defineProps<Props>(), {
  symbol: ''
})

// Emits
const emit = defineEmits<{
  positionClose: [positionId: string]
}>()

// 响应式数据
const viewMode = ref<'all' | 'long' | 'short'>('all')

// 模拟持仓数据
const positions = ref([
  {
    id: '1',
    symbol: 'BTCUSDT',
    side: 'long',
    size: 0.5,
    entryPrice: 50000,
    markPrice: 51000,
    pnl: 500,
    margin: 5000
  },
  {
    id: '2',
    symbol: 'ETHUSDT',
    side: 'short',
    size: 2.0,
    entryPrice: 3000,
    markPrice: 2950,
    pnl: 100,
    margin: 3000
  }
])

// 计算属性
const filteredPositions = computed(() => {
  let filtered = positions.value

  // 按交易对筛选
  if (props.symbol) {
    filtered = filtered.filter(p => p.symbol === props.symbol)
  }

  // 按方向筛选
  if (viewMode.value !== 'all') {
    filtered = filtered.filter(p => p.side === viewMode.value)
  }

  return filtered
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

const formatPnL = (pnl: number) => {
  const sign = pnl >= 0 ? '+' : ''
  return `${sign}${pnl.toFixed(2)} USDT`
}

const closePosition = (positionId: string) => {
  emit('positionClose', positionId)
}
</script>

<style lang="scss" scoped>
.positions-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.positions-header {
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

.positions-table {
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

  &.long {
    color: var(--success-color);
    background: var(--success-bg);
  }

  &.short {
    color: var(--error-color);
    background: var(--error-bg);
  }
}

.pnl-value {
  font-family: var(--font-family-mono);
  font-weight: 600;

  &.positive {
    color: var(--success-color);
  }

  &.negative {
    color: var(--error-color);
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