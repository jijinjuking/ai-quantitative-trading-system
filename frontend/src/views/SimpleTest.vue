<template>
  <div style="padding: 20px; background: #1e2329; color: white; min-height: 100vh;">
    <h1 style="color: #f0b90b; margin-bottom: 20px;">🚀 系统测试页面</h1>
    
    <div style="background: #2b3139; padding: 16px; border-radius: 8px; margin-bottom: 20px;">
      <h2>基础功能测试</h2>
      <p>✅ Vue 3 正常运行</p>
      <p>✅ TypeScript 编译正常</p>
      <p>✅ 路由系统正常</p>
      <p>当前时间: {{ currentTime }}</p>
    </div>

    <div style="background: #2b3139; padding: 16px; border-radius: 8px; margin-bottom: 20px;">
      <h2>API测试</h2>
      <button 
        @click="testAPI" 
        style="background: #f0b90b; color: black; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer; margin-right: 10px;"
      >
        测试市场数据API
      </button>
      <button 
        @click="goToTrading" 
        style="background: #02c076; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;"
      >
        进入交易界面
      </button>
      
      <div v-if="apiResult" style="margin-top: 16px;">
        <h3>API结果:</h3>
        <pre style="background: #0b0e11; padding: 12px; border-radius: 4px; overflow-x: auto; font-size: 12px;">{{ apiResult }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const currentTime = ref('')
const apiResult = ref('')

const updateTime = () => {
  currentTime.value = new Date().toLocaleString()
}

const testAPI = async () => {
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

onMounted(() => {
  updateTime()
  setInterval(updateTime, 1000)
})
</script>