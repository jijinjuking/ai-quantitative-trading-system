<template>
  <div class="trading-chart">
    <div 
      ref="chartContainer" 
      class="chart-container"
      :style="{ height: height }"
    ></div>
    
    <div class="chart-loading" v-if="loading">
      <div class="loading-spinner"></div>
      <span>加载真实K线数据...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import * as echarts from 'echarts'

// Props
interface Props {
  symbol: string
  interval: string
  theme?: 'dark' | 'light'
  height?: string
}

const props = withDefaults(defineProps<Props>(), {
  theme: 'dark',
  height: '100%'
})

// 响应式数据
const chartContainer = ref<HTMLDivElement>()
const loading = ref(true)
let chartInstance: echarts.ECharts | null = null

// 获取真实K线数据
const fetchRealKlineData = async () => {
  try {
    const response = await fetch(`http://localhost:8081/api/v1/klines`)
    const result = await response.json()
    
    if (result.success && result.data) {
      return result.data.map((item: any) => [
        new Date(item.open_time).toISOString().slice(0, 16).replace('T', ' '),
        parseFloat(item.open),
        parseFloat(item.close),
        parseFloat(item.low),
        parseFloat(item.high),
        parseFloat(item.volume)
      ])
    }
  } catch (error) {
    console.error('Failed to fetch real kline data:', error)
  }
  
  // 如果获取失败，返回空数组
  return []
}

// 初始化图表
const initChart = async () => {
  if (!chartContainer.value) return
  
  loading.value = true
  
  // 销毁现有图表
  if (chartInstance) {
    chartInstance.dispose()
  }
  
  // 创建新图表
  chartInstance = echarts.init(chartContainer.value, 'dark')
  
  // 获取真实数据
  const rawData = await fetchRealKlineData()
  
  if (rawData.length === 0) {
    loading.value = false
    return
  }
  
  const dates = rawData.map((item: any) => item[0])
  const klineData = rawData.map((item: any) => [
    item[1], // open
    item[2], // close
    item[3], // low
    item[4]  // high
  ])
  const volumeData = rawData.map((item: any) => item[5])
  
  const option = {
    backgroundColor: '#1e2329',
    animation: false,
    legend: {
      bottom: 10,
      left: 'center',
      data: ['K线', '成交量'],
      textStyle: {
        color: '#848e9c'
      }
    },
    tooltip: {
      trigger: 'axis',
      axisPointer: {
        type: 'cross'
      },
      backgroundColor: 'rgba(43, 49, 57, 0.9)',
      borderColor: '#3c4043',
      textStyle: {
        color: '#eaecef'
      },
      formatter: function (params: any) {
        const data = params[0]
        if (!data) return ''
        
        const kline = data.data
        const volume = volumeData[data.dataIndex]
        
        return `
          <div style="padding: 8px;">
            <div style="margin-bottom: 4px; font-weight: bold;">${props.symbol}</div>
            <div>时间: ${data.name}</div>
            <div>开盘: <span style="color: #eaecef;">${kline[0].toFixed(2)}</span></div>
            <div>收盘: <span style="color: ${kline[1] >= kline[0] ? '#02c076' : '#f84960'};">${kline[1].toFixed(2)}</span></div>
            <div>最高: <span style="color: #eaecef;">${kline[3].toFixed(2)}</span></div>
            <div>最低: <span style="color: #eaecef;">${kline[2].toFixed(2)}</span></div>
            <div>成交量: <span style="color: #848e9c;">${volume.toFixed(2)}</span></div>
          </div>
        `
      }
    },
    axisPointer: {
      link: [
        {
          xAxisIndex: 'all'
        }
      ],
      label: {
        backgroundColor: '#2b3139'
      }
    },
    grid: [
      {
        left: '3%',
        right: '12%',
        top: '5%',
        height: '65%'
      },
      {
        left: '3%',
        right: '12%',
        top: '75%',
        height: '16%'
      }
    ],
    xAxis: [
      {
        type: 'category',
        data: dates,
        scale: true,
        boundaryGap: false,
        axisLine: { onZero: false },
        splitLine: { show: false },
        splitNumber: 20,
        min: 'dataMin',
        max: 'dataMax',
        axisPointer: {
          z: 100
        },
        axisLabel: {
          color: '#848e9c',
          formatter: function (value: string) {
            return value.slice(5, 16) // 显示月-日 时:分
          }
        }
      },
      {
        type: 'category',
        gridIndex: 1,
        data: dates,
        scale: true,
        boundaryGap: false,
        axisLine: { onZero: false },
        axisTick: { show: false },
        splitLine: { show: false },
        axisLabel: { show: false },
        splitNumber: 20,
        min: 'dataMin',
        max: 'dataMax'
      }
    ],
    yAxis: [
      {
        scale: true,
        position: 'right',
        splitArea: {
          show: true,
          areaStyle: {
            color: ['rgba(43, 49, 57, 0.1)', 'rgba(43, 49, 57, 0.3)']
          }
        },
        axisLabel: {
          show: true,
          color: '#848e9c',
          fontSize: 11,
          formatter: function (value: number) {
            return '$' + value.toLocaleString('en-US', { 
              minimumFractionDigits: 0, 
              maximumFractionDigits: 0 
            })
          }
        },
        axisLine: {
          show: true,
          lineStyle: {
            color: '#2b3139'
          }
        },
        axisTick: {
          show: true,
          lineStyle: {
            color: '#2b3139'
          }
        },
        splitLine: {
          show: true,
          lineStyle: {
            color: '#2b3139',
            width: 1,
            type: 'dashed'
          }
        }
      },
      {
        scale: true,
        gridIndex: 1,
        splitNumber: 2,
        axisLabel: { 
          show: false 
        },
        axisLine: { 
          show: false 
        },
        axisTick: { 
          show: false 
        },
        splitLine: { 
          show: false 
        }
      }
    ],
    dataZoom: [
      {
        type: 'inside',
        xAxisIndex: [0, 1],
        start: 80,
        end: 100
      },
      {
        show: true,
        xAxisIndex: [0, 1],
        type: 'slider',
        top: '85%',
        start: 80,
        end: 100,
        backgroundColor: '#2b3139',
        borderColor: '#3c4043',
        fillerColor: 'rgba(240, 185, 11, 0.2)',
        handleStyle: {
          color: '#f0b90b'
        },
        textStyle: {
          color: '#848e9c'
        }
      }
    ],
    series: [
      {
        name: 'K线',
        type: 'candlestick',
        data: klineData,
        itemStyle: {
          color: '#02c076',
          color0: '#f84960',
          borderColor: '#02c076',
          borderColor0: '#f84960'
        },
        emphasis: {
          itemStyle: {
            color: '#03d47c',
            color0: '#ff6b93',
            borderColor: '#03d47c',
            borderColor0: '#ff6b93'
          }
        }
      },
      {
        name: '成交量',
        type: 'bar',
        xAxisIndex: 1,
        yAxisIndex: 1,
        data: volumeData.map((volume: any, index: number) => {
          const kline = klineData[index]
          return {
            value: volume,
            itemStyle: {
              color: kline[1] >= kline[0] ? '#02c076' : '#f84960'
            }
          }
        })
      }
    ]
  }
  
  chartInstance.setOption(option)
  loading.value = false
}

// 更新K线数据
const updateKline = (newData: any) => {
  if (!chartInstance) return
  
  // 这里可以添加实时更新逻辑
  console.log('Updating kline with new data:', newData)
}

// 监听symbol变化
watch(() => props.symbol, () => {
  initChart()
})

// 监听interval变化
watch(() => props.interval, () => {
  initChart()
})

// 启动实时更新
let updateTimer: number | null = null

const startRealTimeUpdate = () => {
  updateTimer = setInterval(async () => {
    if (chartInstance) {
      // 重新获取数据并更新图表
      const rawData = await fetchRealKlineData()
      if (rawData.length > 0) {
        const dates = rawData.map((item: any) => item[0])
        const klineData = rawData.map((item: any) => [
          item[1], // open
          item[2], // close
          item[3], // low
          item[4]  // high
        ])
        const volumeData = rawData.map((item: any) => item[5])
        
        // 更新图表数据
        chartInstance.setOption({
          xAxis: [
            { data: dates },
            { data: dates }
          ],
          series: [
            { data: klineData },
            { 
              data: volumeData.map((volume: any, index: number) => {
                const kline = klineData[index]
                return {
                  value: volume,
                  itemStyle: {
                    color: kline[1] >= kline[0] ? '#02c076' : '#f84960'
                  }
                }
              })
            }
          ]
        })
      }
    }
  }, 5000) // 每5秒更新一次K线图
}

const stopRealTimeUpdate = () => {
  if (updateTimer) {
    clearInterval(updateTimer)
    updateTimer = null
  }
}

// 窗口大小变化时重新调整图表
const handleResize = () => {
  if (chartInstance) {
    chartInstance.resize()
  }
}

// 生命周期
onMounted(() => {
  nextTick(() => {
    initChart()
    startRealTimeUpdate() // 启动实时更新
  })
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  stopRealTimeUpdate() // 停止实时更新
  if (chartInstance) {
    chartInstance.dispose()
  }
  window.removeEventListener('resize', handleResize)
})

// 暴露方法
defineExpose({
  updateKline
})
</script>

<style lang="scss" scoped>
.trading-chart {
  position: relative;
  width: 100%;
  height: 100%;
  background: #1e2329;
  border-radius: 4px;
  overflow: hidden;
}

.chart-container {
  width: 100%;
  height: 100%;
}

.chart-loading {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: #848e9c;
  font-size: 14px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #2b3139;
  border-top: 3px solid #f0b90b;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>