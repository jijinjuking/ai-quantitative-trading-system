<template>
  <div id="app" class="trading-platform">
    <!-- 主题切换器 -->
    <div class="theme-controls">
      <ThemeToggle mode="simple" :animated="true" />
    </div>
    
    <!-- 主要内容区域 -->
    <div class="app-content">
      <router-view />
    </div>
    
    <!-- 连接状态指示器 -->
    <div class="connection-status">
      <div 
        class="connection-indicator"
        :class="connectionStatusClass"
        :title="connectionStatusText"
      ></div>
      <span class="status-text">{{ connectionStatusText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { useWebSocketStore } from '@/stores/websocket'
import ThemeToggle from '@/components/common/ThemeToggle.vue'

// Composables
const { appliedTheme } = useTheme()
const wsStore = useWebSocketStore()

// 计算属性
const connectionStatusClass = computed(() => {
  switch (wsStore.connectionStatus) {
    case 'connected':
      return 'connected'
    case 'connecting':
      return 'connecting'
    case 'disconnected':
      return 'disconnected'
    case 'error':
      return 'disconnected'
    default:
      return 'disconnected'
  }
})

const connectionStatusText = computed(() => wsStore.connectionStatusText)

// 生命周期
onMounted(() => {
  console.log('App mounted successfully')
  // WebSocket连接将在需要时手动初始化
})
</script>

<style lang="scss">
// 导入全局样式
@use '@/styles/global.scss';

.trading-platform {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  color: var(--text-primary);
  overflow: hidden;
}

.theme-controls {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: var(--z-fixed);
}

.app-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.connection-status {
  position: fixed;
  bottom: 16px;
  right: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-full);
  box-shadow: var(--shadow-md);
  z-index: var(--z-fixed);
  
  .connection-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    
    &.connected {
      background: var(--success-color);
      box-shadow: 0 0 6px var(--success-color);
      animation: pulse 2s infinite;
    }
    
    &.connecting {
      background: var(--warning-color);
      animation: pulse 1s infinite;
    }
    
    &.disconnected {
      background: var(--error-color);
      animation: blink 1s infinite;
    }
  }
  
  .status-text {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    font-weight: var(--font-weight-medium);
  }
}

// 响应式设计
@media (max-width: 768px) {
  .theme-controls {
    top: 12px;
    right: 12px;
  }
  
  .connection-status {
    bottom: 12px;
    right: 12px;
    padding: 6px 10px;
    
    .status-text {
      display: none;
    }
  }
}
</style>