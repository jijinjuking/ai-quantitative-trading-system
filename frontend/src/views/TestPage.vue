<template>
  <div class="test-page">
    <h1>🚀 Rust微服务交易系统</h1>
    <div class="test-section">
      <h2>✅ 系统状态检查</h2>
      <p>如果你能看到这个页面，说明前端基础功能正常</p>
      
      <div class="status-grid">
        <div class="status-card">
          <h3>前端服务</h3>
          <div class="status-indicator success">✅ 正常运行</div>
          <p>端口: 3000</p>
        </div>
        
        <div class="status-card">
          <h3>API网关</h3>
          <div class="status-indicator">🔄 检查中...</div>
          <p>端口: 8080</p>
        </div>
        
        <div class="status-card">
          <h3>市场数据服务</h3>
          <div class="status-indicator">🔄 检查中...</div>
          <p>端口: 8081</p>
        </div>
      </div>
      
      <div class="action-buttons">
        <button @click="testMarketData" class="test-btn">测试市场数据API</button>
        <button @click="goToTrading" class="trading-btn">进入交易界面</button>
      </div>
      
      <div v-if="apiResult" class="result-section">
        <h3>API测试结果</h3>
        <pre>{{ apiResult }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const apiResult = ref('')

const testMarketData = async () => {
  try {
    const response = await fetch('http://localhost:8081/api/v1/tickers')
    const data = await response.json()
    apiResult.value = JSON.stringify(data, null, 2)
  } catch (error) {
    apiResult.value = `API测试失败: ${error}`
  }
}

const goToTrading = () => {
  router.push('/trading')
}
</script>

<style lang="scss" scoped>
.test-page {
  padding: 20px;
  max-width: 800px;
  margin: 0 auto;
  background: #1e2329;
  color: #eaecef;
  min-height: 100vh;

  h1, h2, h3 {
    color: #f0b90b;
    margin-bottom: 16px;
  }

  .test-section {
    margin-bottom: 32px;
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin: 24px 0;
  }

  .status-card {
    background: #2b3139;
    padding: 16px;
    border-radius: 8px;
    text-align: center;

    h3 {
      margin-bottom: 12px;
      font-size: 16px;
    }

    .status-indicator {
      padding: 8px 12px;
      border-radius: 4px;
      margin: 8px 0;
      font-weight: bold;

      &.success {
        background: rgba(2, 192, 118, 0.2);
        color: #02c076;
      }
    }

    p {
      margin: 8px 0 0 0;
      color: #848e9c;
      font-size: 14px;
    }
  }

  .action-buttons {
    display: flex;
    gap: 16px;
    margin: 24px 0;
    justify-content: center;

    button {
      padding: 12px 24px;
      border: none;
      border-radius: 6px;
      cursor: pointer;
      font-weight: 600;
      transition: all 0.2s;

      &.test-btn {
        background: #f0b90b;
        color: #000;

        &:hover {
          background: #fcd535;
        }
      }

      &.trading-btn {
        background: #02c076;
        color: #fff;

        &:hover {
          background: #03d47c;
        }
      }
    }
  }

  .result-section {
    background: #2b3139;
    padding: 16px;
    border-radius: 8px;
    margin-top: 24px;

    pre {
      background: #0b0e11;
      padding: 12px;
      border-radius: 4px;
      overflow-x: auto;
      font-size: 12px;
      margin-top: 12px;
    }
  }
}
</style>