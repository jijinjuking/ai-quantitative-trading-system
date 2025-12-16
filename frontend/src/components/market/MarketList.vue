<template>
  <div class="market-list">
    <!-- 头部工具栏 -->
    <div class="market-header">
      <div class="header-left">
        <h3>市场</h3>
      </div>
      <div class="header-right">
        <el-input
          v-model="searchKeyword"
          placeholder="搜索交易对"
          prefix-icon="el-icon-search"
          size="small"
          clearable
          @input="onSearch"
        />
        <el-button size="small" @click="refreshData">
          <i class="el-icon-refresh"></i>
        </el-button>
      </div>
    </div>

    <!-- 分类标签 -->
    <div class="market-categories">
      <el-tabs v-model="activeCategory" @tab-change="onCategoryChange">
        <el-tab-pane 
          v-for="category in categories"
          :key="category.value"
          :label="category.label"
          :name="category.value"
        >
          <!-- 排序选项 -->
          <div class="sort-options">
            <el-button-group size="small">
              <el-button 
                :type="sortBy === 'volume' ? 'primary' : 'default'"
                @click="setSortBy('volume')"
              >
                成交量
              </el-button>
              <el-button 
                :type="sortBy === 'change' ? 'primary' : 'default'"
                @click="setSortBy('change')"
              >
                涨跌幅
              </el-button>
              <el-button 
                :type="sortBy === 'price' ? 'primary' : 'default'"
                @click="setSortBy('price')"
              >
                价格
              </el-button>
              <el-button 
                :type="sortBy === 'name' ? 'primary' : 'default'"
                @click="setSortBy('name')"
              >
                名称
              </el-button>
            </el-button-group>
            
            <el-button 
              size="small"
              @click="toggleSortOrder"
              :icon="sortOrder === 'desc' ? 'el-icon-sort-down' : 'el-icon-sort-up'"
            />
          </div>

          <!-- 市场列表 -->
          <div class="market-content">
            <el-table
              :data="filteredSymbols"
              :loading="isLoading"
              size="small"
              class="market-table"
              @row-click="onSymbolClick"
              :row-class-name="getRowClassName"
            >
              <!-- 自选 -->
              <el-table-column width="40" fixed="left">
                <template #default="{ row }">
                  <el-button
                    text
                    :icon="isInWatchlist(row.symbol) ? 'el-icon-star-on' : 'el-icon-star-off'"
                    :class="{ 'favorited': isInWatchlist(row.symbol) }"
                    @click.stop="toggleWatchlist(row.symbol)"
                  />
                </template>
              </el-table-column>

              <!-- 交易对 -->
              <el-table-column prop="symbol" label="交易对" width="120" fixed="left">
                <template #default="{ row }">
                  <div class="symbol-cell">
                    <span class="base-asset">{{ row.baseAsset }}</span>
                    <span class="quote-asset">/{{ row.quoteAsset }}</span>
                  </div>
                </template>
              </el-table-column>

              <!-- 最新价格 -->
              <el-table-column prop="price" label="最新价" width="120">
                <template #default="{ row }">
                  <div class="price-cell">
                    <span class="price" :class="getPriceChangeClass(row.priceChangePercent)">
                      {{ formatPrice(row.price) }}
                    </span>
                    <span class="price-usd" v-if="row.usdPrice">
                      ≈ ${{ formatPrice(row.usdPrice) }}
                    </span>
                  </div>
                </template>
              </el-table-column>

              <!-- 24h涨跌幅 -->
              <el-table-column prop="priceChangePercent" label="24h涨跌幅" width="100">
                <template #default="{ row }">
                  <div class="change-cell">
                    <span class="change-percent" :class="getPriceChangeClass(row.priceChangePercent)">
                      {{ formatPercent(row.priceChangePercent) }}
                    </span>
                    <span class="change-amount" :class="getPriceChangeClass(row.priceChangePercent)">
                      {{ formatPriceChange(row.priceChange) }}
                    </span>
                  </div>
                </template>
              </el-table-column>

              <!-- 24h最高价 -->
              <el-table-column prop="high" label="24h最高" width="100">
                <template #default="{ row }">
                  <span class="high-price">{{ formatPrice(row.high) }}</span>
                </template>
              </el-table-column>

              <!-- 24h最低价 -->
              <el-table-column prop="low" label="24h最低" width="100">
                <template #default="{ row }">
                  <span class="low-price">{{ formatPrice(row.low) }}</span>
                </template>
              </el-table-column>

              <!-- 24h成交量 -->
              <el-table-column prop="volume" label="24h成交量" width="120">
                <template #default="{ row }">
                  <div class="volume-cell">
                    <span class="volume">{{ formatVolume(row.volume) }}</span>
                    <span class="volume-quote">{{ formatVolume(row.quoteVolume) }} {{ row.quoteAsset }}</span>
                  </div>
                </template>
              </el-table-column>

              <!-- 市值 -->
              <el-table-column prop="marketCap" label="市值" width="100" v-if="showMarketCap">
                <template #default="{ row }">
                  <span v-if="row.marketCap">{{ formatMarketCap(row.marketCap) }}</span>
                  <span v-else>-</span>
                </template>
              </el-table-column>

              <!-- 操作 -->
              <el-table-column label="操作" width="100" fixed="right">
                <template #default="{ row }">
                  <div class="action-buttons">
                    <el-button 
                      size="small" 
                      type="primary"
                      @click.stop="goToTrading(row.symbol)"
                    >
                      交易
                    </el-button>
                  </div>
                </template>
              </el-table-column>
            </el-table>

            <!-- 加载更多 -->
            <div class="load-more" v-if="hasMore && !isLoading">
              <el-button @click="loadMore" :loading="isLoadingMore">
                加载更多
              </el-button>
            </div>

            <!-- 空状态 -->
            <div v-if="filteredSymbols.length === 0 && !isLoading" class="empty-state">
              <el-empty description="未找到相关交易对">
                <el-button @click="clearSearch">清除搜索</el-button>
              </el-empty>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <!-- 市场统计 -->
    <div class="market-stats" v-if="showStats">
      <div class="stats-row">
        <div class="stat-item">
          <span class="label">总市值</span>
          <span class="value">{{ formatMarketCap(marketStats.totalMarketCap) }}</span>
        </div>
        <div class="stat-item">
          <span class="label">24h成交量</span>
          <span class="value">{{ formatVolume(marketStats.totalVolume24h) }}</span>
        </div>
        <div class="stat-item">
          <span class="label">上涨</span>
          <span class="value positive">{{ marketStats.gainers }}</span>
        </div>
        <div class="stat-item">
          <span class="label">下跌</span>
          <span class="value negative">{{ marketStats.losers }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useMarketStore } from '@/stores/market'
import { useUserStore } from '@/stores/user'
import { useWebSocketStore } from '@/stores/websocket'

// Props
interface Props {
  showStats?: boolean
  showMarketCap?: boolean
  pageSize?: number
}

const props = withDefaults(defineProps<Props>(), {
  showStats: true,
  showMarketCap: true,
  pageSize: 50
})

// Emits
const emit = defineEmits<{
  symbolSelect: [symbol: string]
  symbolFavorite: [symbol: string, favorited: boolean]
}>()

// 状态管理
const marketStore = useMarketStore()
const userStore = useUserStore()
const wsStore = useWebSocketStore()
const router = useRouter()

// 响应式数据
const activeCategory = ref('USDT')
const searchKeyword = ref('')
const sortBy = ref('volume')
const sortOrder = ref<'asc' | 'desc'>('desc')
const isLoading = ref(false)
const isLoadingMore = ref(false)
const currentPage = ref(1)
const hasMore = ref(true)

// 分类配置
const categories = [
  { label: 'USDT', value: 'USDT' },
  { label: 'BTC', value: 'BTC' },
  { label: 'ETH', value: 'ETH' },
  { label: '热门', value: 'HOT' },
  { label: '新币', value: 'NEW' },
  { label: 'DeFi', value: 'DEFI' },
  { label: 'NFT', value: 'NFT' }
]

// 计算属性
const allSymbols = computed(() => {
  return marketStore.marketSymbols
})

const categorySymbols = computed(() => {
  return marketStore.getSymbolsByCategory(activeCategory.value)
})

const searchedSymbols = computed(() => {
  if (!searchKeyword.value) {
    return categorySymbols.value
  }
  
  return marketStore.searchSymbols(searchKeyword.value)
})

const sortedSymbols = computed(() => {
  const symbols = [...searchedSymbols.value]
  
  symbols.sort((a, b) => {
    let aValue, bValue
    
    switch (sortBy.value) {
      case 'volume':
        aValue = a.quoteVolume || 0
        bValue = b.quoteVolume || 0
        break
      case 'change':
        aValue = a.priceChangePercent || 0
        bValue = b.priceChangePercent || 0
        break
      case 'price':
        aValue = a.price || 0
        bValue = b.price || 0
        break
      case 'name':
        aValue = a.symbol
        bValue = b.symbol
        return sortOrder.value === 'desc' 
          ? bValue.localeCompare(aValue)
          : aValue.localeCompare(bValue)
      default:
        return 0
    }
    
    return sortOrder.value === 'desc' ? bValue - aValue : aValue - bValue
  })
  
  return symbols
})

const filteredSymbols = computed(() => {
  // 分页处理
  const endIndex = currentPage.value * props.pageSize
  return sortedSymbols.value.slice(0, endIndex)
})

const marketStats = computed(() => {
  return marketStore.marketStats
})

const watchlistSymbols = computed(() => {
  return userStore.watchlistSymbols
})

// 生命周期
onMounted(() => {
  loadMarketData()
  setupWebSocketSubscription()
})

// 加载市场数据
const loadMarketData = async () => {
  try {
    isLoading.value = true
    await marketStore.loadMarketData()
  } catch (error) {
    ElMessage.error('加载市场数据失败: ' + error.message)
  } finally {
    isLoading.value = false
  }
}

// WebSocket订阅
const setupWebSocketSubscription = () => {
  // 订阅价格更新
  wsStore.subscribe('ticker', null, (data) => {
    marketStore.updateTicker(data.symbol, data)
  })
}

// 事件处理
const onSymbolClick = (row: any) => {
  emit('symbolSelect', row.symbol)
}

const onCategoryChange = (category: string) => {
  activeCategory.value = category
  currentPage.value = 1
  hasMore.value = true
}

const onSearch = (keyword: string) => {
  searchKeyword.value = keyword
  currentPage.value = 1
  hasMore.value = true
}

const setSortBy = (field: string) => {
  if (sortBy.value === field) {
    toggleSortOrder()
  } else {
    sortBy.value = field
    sortOrder.value = 'desc'
  }
  currentPage.value = 1
}

const toggleSortOrder = () => {
  sortOrder.value = sortOrder.value === 'desc' ? 'asc' : 'desc'
}

const toggleWatchlist = async (symbol: string) => {
  try {
    await userStore.toggleWatchlist(symbol)
    const favorited = userStore.watchlistSymbols.includes(symbol)
    emit('symbolFavorite', symbol, favorited)
    
    ElMessage.success(favorited ? '已添加到自选' : '已从自选移除')
  } catch (error) {
    ElMessage.error('操作失败: ' + error.message)
  }
}

const goToTrading = (symbol: string) => {
  router.push(`/trading?symbol=${symbol}`)
}

const refreshData = async () => {
  await loadMarketData()
}

const loadMore = async () => {
  try {
    isLoadingMore.value = true
    currentPage.value++
    
    // 检查是否还有更多数据
    if (filteredSymbols.value.length >= sortedSymbols.value.length) {
      hasMore.value = false
    }
  } catch (error) {
    ElMessage.error('加载失败: ' + error.message)
  } finally {
    isLoadingMore.value = false
  }
}

const clearSearch = () => {
  searchKeyword.value = ''
}

// 工具函数
const isInWatchlist = (symbol: string): boolean => {
  return watchlistSymbols.value.includes(symbol)
}

const getPriceChangeClass = (change: number): string => {
  if (change > 0) return 'positive'
  if (change < 0) return 'negative'
  return 'neutral'
}

const getRowClassName = ({ row }: { row: any }): string => {
  return isInWatchlist(row.symbol) ? 'favorited-row' : ''
}

const formatPrice = (price: number): string => {
  if (price >= 1) {
    return price.toFixed(2)
  } else if (price >= 0.01) {
    return price.toFixed(4)
  } else {
    return price.toFixed(8)
  }
}

const formatPercent = (percent: number): string => {
  const sign = percent >= 0 ? '+' : ''
  return `${sign}${percent.toFixed(2)}%`
}

const formatPriceChange = (change: number): string => {
  const sign = change >= 0 ? '+' : ''
  return `${sign}${formatPrice(Math.abs(change))}`
}

const formatVolume = (volume: number): string => {
  if (volume >= 1e9) {
    return (volume / 1e9).toFixed(2) + 'B'
  } else if (volume >= 1e6) {
    return (volume / 1e6).toFixed(2) + 'M'
  } else if (volume >= 1e3) {
    return (volume / 1e3).toFixed(2) + 'K'
  }
  return volume.toFixed(2)
}

const formatMarketCap = (marketCap: number | undefined): string => {
  if (!marketCap || isNaN(marketCap)) {
    return '--'
  }
  if (marketCap >= 1e12) {
    return '$' + (marketCap / 1e12).toFixed(2) + 'T'
  } else if (marketCap >= 1e9) {
    return '$' + (marketCap / 1e9).toFixed(2) + 'B'
  } else if (marketCap >= 1e6) {
    return '$' + (marketCap / 1e6).toFixed(2) + 'M'
  }
  return '$' + marketCap.toFixed(2)
}

// 监听搜索关键词变化
watch(searchKeyword, () => {
  currentPage.value = 1
  hasMore.value = true
})

// 监听排序变化
watch([sortBy, sortOrder], () => {
  currentPage.value = 1
  hasMore.value = true
})
</script>

<style lang="scss" scoped>
.market-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
}

.market-header {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  
  .header-left {
    h3 {
      margin: 0;
      font-size: 16px;
      font-weight: 600;
      color: var(--text-primary);
    }
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    
    .el-input {
      width: 200px;
    }
  }
}

.market-categories {
  flex: 1;
  overflow: hidden;
  
  :deep(.el-tabs) {
    height: 100%;
    display: flex;
    flex-direction: column;
    
    .el-tabs__header {
      margin: 0;
      background: var(--bg-secondary);
      border-bottom: 1px solid var(--border-color);
      
      .el-tabs__nav-wrap {
        padding: 0 16px;
      }
    }
    
    .el-tabs__content {
      flex: 1;
      overflow: hidden;
      padding: 0;
      
      .el-tab-pane {
        height: 100%;
        display: flex;
        flex-direction: column;
      }
    }
  }
  
  .sort-options {
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-color);
  }
  
  .market-content {
    flex: 1;
    overflow: hidden;
    
    .market-table {
      height: 100%;
      
      :deep(.el-table__body-wrapper) {
        overflow-y: auto;
      }
      
      .symbol-cell {
        .base-asset {
          font-weight: 600;
          color: var(--text-primary);
        }
        
        .quote-asset {
          color: var(--text-secondary);
          font-size: 12px;
        }
      }
      
      .price-cell {
        display: flex;
        flex-direction: column;
        
        .price {
          font-weight: 600;
          
          &.positive {
            color: var(--success-color);
          }
          
          &.negative {
            color: var(--error-color);
          }
        }
        
        .price-usd {
          font-size: 11px;
          color: var(--text-secondary);
        }
      }
      
      .change-cell {
        display: flex;
        flex-direction: column;
        
        .change-percent {
          font-weight: 600;
          
          &.positive {
            color: var(--success-color);
          }
          
          &.negative {
            color: var(--error-color);
          }
        }
        
        .change-amount {
          font-size: 11px;
          
          &.positive {
            color: var(--success-color);
          }
          
          &.negative {
            color: var(--error-color);
          }
        }
      }
      
      .volume-cell {
        display: flex;
        flex-direction: column;
        
        .volume {
          font-weight: 600;
        }
        
        .volume-quote {
          font-size: 11px;
          color: var(--text-secondary);
        }
      }
      
      .high-price {
        color: var(--success-color);
      }
      
      .low-price {
        color: var(--error-color);
      }
      
      .favorited {
        color: var(--warning-color);
      }
      
      .action-buttons {
        .el-button {
          padding: 4px 12px;
        }
      }
    }
    
    .favorited-row {
      background: var(--bg-tertiary);
    }
    
    .load-more {
      padding: 16px;
      text-align: center;
      background: var(--bg-secondary);
      border-top: 1px solid var(--border-color);
    }
    
    .empty-state {
      height: 200px;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }
}

.market-stats {
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  
  .stats-row {
    display: flex;
    justify-content: space-around;
    
    .stat-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      
      .label {
        font-size: 11px;
        color: var(--text-secondary);
        margin-bottom: 4px;
      }
      
      .value {
        font-size: 14px;
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
}

// 响应式设计
@media (max-width: 768px) {
  .market-header {
    padding: 0 12px;
    
    .header-right {
      .el-input {
        width: 150px;
      }
    }
  }
  
  .sort-options {
    padding: 0 12px;
    
    .el-button-group {
      .el-button {
        padding: 4px 8px;
        font-size: 12px;
      }
    }
  }
  
  .market-table {
    :deep(.el-table__header) {
      .cell {
        font-size: 11px;
      }
    }
    
    :deep(.el-table__body) {
      .cell {
        font-size: 11px;
      }
    }
    
    .action-buttons {
      .el-button {
        padding: 2px 8px;
        font-size: 11px;
      }
    }
  }
  
  .market-stats {
    padding: 8px 12px;
    
    .stat-item {
      .label {
        font-size: 10px;
      }
      
      .value {
        font-size: 12px;
      }
    }
  }
}
</style>