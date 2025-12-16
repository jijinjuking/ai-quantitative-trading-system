<template>
  <div class="system-status">
    <div class="status-header">
      <h3>系统状态</h3>
      <button @click="refreshStatus" :disabled="isRefreshing" class="refresh-btn">
        <i :class="['refresh-icon', { spinning: isRefreshing }]">🔄</i>
        刷新
      </button>
    </div>

    <div class="status-grid">
      <!-- 网关状态 -->
      <div class="status-card">
        <div class="status-header-card">
          <span class="service-name">API网关</span>
          <span :class="['status-indicator', gatewayStatus.status]">
            {{ getStatusText(gatewayStatus.status) }}
          </span>
        </div>
        <div class="status-details">
          <div class="detail-item">
            <span>地址:</span>
            <span>localhost:8080</span>
          </div>
          <div class="detail-item">
            <span>响应时间:</span>
            <span>{{ gatewayStatus.responseTime }}ms</span>
          </div>
        </div>
      </div>

      <!-- 市场数据服务状态 -->
      <div class="status-card">
        <div class="status-header-card">
          <span class="service-name">市场数据服务</span>
          <span :class="['status-indicator', marketDataStatus.status]">
            {{ getStatusText(marketDataStatus.status) }}
          </span>
        </div>
        <div class="status-details">
          <div class="detail-item">
            <span>地址:</span>
            <span>localhost:8083</span>
          </div>
          <div class="detail-item">
            <span>WebSocket:</span>
            <span :class="['ws-status', websocketStatus]">
              {{ websocketStatus === 'connected' ? '已连接' : '未连接' }}
            </span>
          </div>
        </div>
      </div>

      <!-- 交易引擎状态 -->
      <div class="status-card">
        <div class="status-header-card">
          <span class="service-name">交易引擎</span>
          <span :class="['status-indicator', tradingStatus.status]">
            {{ getStatusText(tradingStatus.status) }}
          </span>
        </div>
        <div class="status-details">
          <div class="detail-item">
            <span>地址:</span>
            <span>localhost:8082</span>
          </div>
          <div class="detail-item">
            <span>订单处理:</span>
            <span>{{ tradingStatus.orderCount || 0 }}</span>
          </div>
        </div>
      </div>

      <!-- 数据库状态 -->
      <div class="status-card">
        <div class="status-header-card">
          <span class="service-name">数据存储</span>
          <span :class="['status-indicator', databaseStatus.status]">
            {{ getStatusText(databaseStatus.status) }}
          </span>
        </div>
        <div class="status-details">
          <div class="detail-item">
            <span>ClickHouse:</span>
            <span :class="['db-status', databaseStatus.clickhouse]">
              {{ databaseStatus.clickhouse === 'healthy' ? '正常' : '异常' }}
            </span>
          </div>
          <div class="detail-item">
            <span>Redis:</span>
            <span :class="['db-status', databaseStatus.redis]">
              {{ databaseStatus.redis === 'healthy' ? '正常' : '异常' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- WebSocket连接详情 -->
    <div class="websocket-details" v-if="websocketDetails">
      <h4>WebSocket连接详情</h4>
      <div class="ws-info">
        <div class="ws-item">
          <span>连接状态:</span>
          <span :class="['ws-status', websocketDetails.status]">
            {{ websocketDetails.statusText }}
          </span>
        </div>
        <div class="ws-item">
          <span>订阅数量:</span>
          <span>{{ websocketDetails.subscriptionCount }}</span>
        </div>
        <div class="ws-item">
          <span>重连次数:</span>
          <span>{{ websocketDetails.reconnectAttempts }}</span>
        </div>
        <div class="ws-item" v-if="websocketDetails.lastError">
          <span>最后错误:</span>
          <span class="error-text">{{ websocketDetails.lastError }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { healthApi } from '@/utils/api'
import { useWebSocketStore } from '@/stores/websocket'

interface ServiceStatus {
  status: 'healthy' | 'unhealthy' | 'unknown'
  responseTime: number
  lastCheck: Date
}

interface DatabaseStatus {
  status: 'healthy' | 'unhealthy' | 'unknown'
  clickhouse: 'healthy' | 'unhealthy' | 'unknown'
  redis: 'healthy' | 'unhealthy' | 'unknown'
}

// 状态数据
const isRefreshing = ref(false)
const gatewayStatus = ref<ServiceStatus>({
  status: 'unknown',
  responseTime: 0,
  lastCheck: new Date()
})

const marketDataStatus = ref<ServiceStatus>({
  status: 'unknown',
  responseTime: 0,
  lastCheck: new Date()
})

const tradingStatus = ref<ServiceStatus & { orderCount?: number }>({
  status: 'unknown',
  responseTime: 0,
  lastCheck: new Date()
})

const databaseStatus = ref<DatabaseStatus>({
  status: 'unknown',
  clickhouse: 'unknown',
  redis: 'unknown'
})

// WebSocket状态
const websocketStore = useWebSocketStore()
const websocketStatus = ref<'connected' | 'disconnected'>('disconnected')
const websocketDetails = ref<any>(null)

// 定时器
let statusCheckInterval: NodeJS.Timeout | null = null

// 获取状态文本
const getStatusText = (status: string) => {
  switch (status) {
    case 'healthy':
      return '正常'
    case 'unhealthy':
      return '异常'
    case 'unknown':
    default:
      return '未知'
  }
}

// 检查网关状态
const checkGatewayStatus = async () => {
  try {
    const startTime = Date.now()
    await healthApi.gatewayHealth()
    const responseTime = Date.now() - startTime
    
    gatewayStatus.value = {
      status: 'healthy',
      responseTime,
      lastCheck: new Date()
    }
  } catch (error) {
    gatewayStatus.value = {
      status: 'unhealthy',
      responseTime: 0,
      lastCheck: new Date()
    }
  }
}

// 检查服务状态
const checkServicesStatus = async () => {
  try {
    const response = await healthApi.serviceStatus()
    const services = response.data

    // 更新市场数据服务状态
    const marketDataService = services.find((s: any) => s.name === 'market-data')
    if (marketDataService) {
      marketDataStatus.value = {
        status: marketDataService.healthy ? 'healthy' : 'unhealthy',
        responseTime: marketDataService.responseTime || 0,
        lastCheck: new Date()
      }
    }

    // 更新交易引擎状态
    const tradingService = services.find((s: any) => s.name === 'trading')
    if (tradingService) {
      tradingStatus.value = {
        status: tradingService.healthy ? 'healthy' : 'unhealthy',
        responseTime: tradingService.responseTime || 0,
        lastCheck: new Date(),
        orderCount: tradingService.orderCount
      }
    }

  } catch (error) {
    console.error('Failed to check services status:', error)
  }
}

// 检查WebSocket状态
const checkWebSocketStatus = async () => {
  try {
    const response = await healthApi.websocketStats()
    const stats = response.data

    websocketDetails.value = {
      status: websocketStore.connectionStatus,
      statusText: websocketStore.connectionStatusText,
      subscriptionCount: websocketStore.subscriptionCount,
      reconnectAttempts: websocketStore.reconnectAttempts,
      lastError: websocketStore.lastError,
      activeConnections: stats.activeConnections,
      totalMessages: stats.totalMessages
    }

    websocketStatus.value = websocketStore.isConnected ? 'connected' : 'disconnected'
  } catch (error) {
    console.error('Failed to check WebSocket status:', error)
  }
}

// 模拟数据库状态检查
const checkDatabaseStatus = async () => {
  // 这里应该通过API检查数据库状态
  // 暂时使用模拟数据
  databaseStatus.value = {
    status: 'healthy',
    clickhouse: 'healthy',
    redis: 'healthy'
  }
}

// 刷新所有状态
const refreshStatus = async () => {
  if (isRefreshing.value) return

  isRefreshing.value = true
  try {
    await Promise.all([
      checkGatewayStatus(),
      checkServicesStatus(),
      checkWebSocketStatus(),
      checkDatabaseStatus()
    ])
  } catch (error) {
    console.error('Failed to refresh status:', error)
  } finally {
    isRefreshing.value = false
  }
}

// 生命周期
onMounted(() => {
  refreshStatus()
  
  // 每30秒检查一次状态
  statusCheckInterval = setInterval(refreshStatus, 30000)
})

onUnmounted(() => {
  if (statusCheckInterval) {
    clearInterval(statusCheckInterval)
  }
})
</script>

<style lang="scss" scoped>
.system-status {
  padding: 20px;
  background: #1e2329;
  border-radius: 8px;
  color: #eaecef;

  .status-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;

    h3 {
      margin: 0;
      color: #eaecef;
    }

    .refresh-btn {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 8px 16px;
      background: #2b3139;
      border: 1px solid #474d57;
      border-radius: 4px;
      color: #eaecef;
      cursor: pointer;
      transition: all 0.2s;

      &:hover:not(:disabled) {
        background: #474d57;
      }

      &:disabled {
        opacity: 0.6;
        cursor: not-allowed;
      }

      .refresh-icon {
        display: inline-block;
        transition: transform 0.5s;

        &.spinning {
          animation: spin 1s linear infinite;
        }
      }
    }
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 16px;
    margin-bottom: 20px;
  }

  .status-card {
    background: #2b3139;
    border: 1px solid #474d57;
    border-radius: 6px;
    padding: 16px;

    .status-header-card {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;

      .service-name {
        font-weight: 600;
        color: #eaecef;
      }

      .status-indicator {
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 12px;
        font-weight: 500;

        &.healthy {
          background: rgba(2, 192, 118, 0.2);
          color: #02c076;
        }

        &.unhealthy {
          background: rgba(248, 73, 96, 0.2);
          color: #f84960;
        }

        &.unknown {
          background: rgba(234, 236, 239, 0.2);
          color: #848e9c;
        }
      }
    }

    .status-details {
      .detail-item {
        display: flex;
        justify-content: space-between;
        margin-bottom: 8px;
        font-size: 12px;

        &:last-child {
          margin-bottom: 0;
        }

        span:first-child {
          color: #848e9c;
        }

        span:last-child {
          color: #eaecef;
        }

        .ws-status, .db-status {
          &.connected, &.healthy {
            color: #02c076;
          }

          &.disconnected, &.unhealthy {
            color: #f84960;
          }

          &.unknown {
            color: #848e9c;
          }
        }
      }
    }
  }

  .websocket-details {
    background: #2b3139;
    border: 1px solid #474d57;
    border-radius: 6px;
    padding: 16px;

    h4 {
      margin: 0 0 12px 0;
      color: #eaecef;
      font-size: 14px;
    }

    .ws-info {
      .ws-item {
        display: flex;
        justify-content: space-between;
        margin-bottom: 8px;
        font-size: 12px;

        &:last-child {
          margin-bottom: 0;
        }

        span:first-child {
          color: #848e9c;
        }

        span:last-child {
          color: #eaecef;
        }

        .ws-status {
          &.connected {
            color: #02c076;
          }

          &.disconnected, &.error {
            color: #f84960;
          }

          &.connecting {
            color: #fcd535;
          }
        }

        .error-text {
          color: #f84960;
          max-width: 200px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }
      }
    }
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>