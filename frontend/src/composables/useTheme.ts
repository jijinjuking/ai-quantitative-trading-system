import { ref, computed, watch, onMounted } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'auto' | 'high-contrast'

// 主题状态
const currentTheme = ref<ThemeMode>('dark')
const systemTheme = ref<'light' | 'dark'>('dark')

// 主题配置
const themeConfig = {
  light: {
    name: '浅色主题',
    icon: 'el-icon-sunny',
    description: '适合白天使用的明亮主题'
  },
  dark: {
    name: '深色主题', 
    icon: 'el-icon-moon',
    description: '专业交易员偏好的深色主题'
  },
  auto: {
    name: '自动切换',
    icon: 'el-icon-magic-stick',
    description: '根据系统设置自动切换'
  },
  'high-contrast': {
    name: '高对比度',
    icon: 'el-icon-view',
    description: '高对比度主题，提升可读性'
  }
}

export const useTheme = () => {
  // 计算实际应用的主题
  const appliedTheme = computed(() => {
    if (currentTheme.value === 'auto') {
      return systemTheme.value
    }
    return currentTheme.value
  })

  // 检测系统主题
  const detectSystemTheme = () => {
    if (typeof window !== 'undefined') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      systemTheme.value = mediaQuery.matches ? 'dark' : 'light'
      
      // 监听系统主题变化
      mediaQuery.addEventListener('change', (e) => {
        systemTheme.value = e.matches ? 'dark' : 'light'
      })
    }
  }

  // 应用主题到DOM
  const applyTheme = (theme: string) => {
    if (typeof document !== 'undefined') {
      // 移除所有主题类
      document.documentElement.removeAttribute('data-theme')
      
      // 应用新主题
      if (theme !== 'dark') {
        document.documentElement.setAttribute('data-theme', theme)
      }
      
      // 更新meta标签颜色
      updateMetaThemeColor(theme)
      
      // 触发主题变化事件
      window.dispatchEvent(new CustomEvent('theme-change', { 
        detail: { theme } 
      }))
    }
  }

  // 更新meta主题颜色
  const updateMetaThemeColor = (theme: string) => {
    const metaThemeColor = document.querySelector('meta[name="theme-color"]')
    if (metaThemeColor) {
      const colors = {
        light: '#ffffff',
        dark: '#0a0e1a',
        'high-contrast': '#000000'
      }
      metaThemeColor.setAttribute('content', colors[theme] || colors.dark)
    }
  }

  // 设置主题
  const setTheme = (theme: ThemeMode) => {
    currentTheme.value = theme
    
    // 保存到本地存储
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('trading-platform-theme', theme)
    }
  }

  // 切换主题
  const toggleTheme = () => {
    const themes: ThemeMode[] = ['light', 'dark', 'auto', 'high-contrast']
    const currentIndex = themes.indexOf(currentTheme.value)
    const nextIndex = (currentIndex + 1) % themes.length
    setTheme(themes[nextIndex])
  }

  // 从本地存储加载主题
  const loadThemeFromStorage = () => {
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('trading-platform-theme') as ThemeMode
      if (savedTheme && Object.keys(themeConfig).includes(savedTheme)) {
        currentTheme.value = savedTheme
      }
    }
  }

  // 获取主题信息
  const getThemeInfo = (theme: ThemeMode) => {
    return themeConfig[theme]
  }

  // 获取所有可用主题
  const getAvailableThemes = () => {
    return Object.entries(themeConfig).map(([key, config]) => ({
      value: key as ThemeMode,
      ...config
    }))
  }

  // 检查是否为深色主题
  const isDark = computed(() => {
    return appliedTheme.value === 'dark' || appliedTheme.value === 'high-contrast'
  })

  // 检查是否为浅色主题
  const isLight = computed(() => {
    return appliedTheme.value === 'light'
  })

  // 检查是否为高对比度主题
  const isHighContrast = computed(() => {
    return appliedTheme.value === 'high-contrast'
  })

  // 获取主题CSS变量
  const getThemeVariable = (variable: string) => {
    if (typeof document !== 'undefined') {
      return getComputedStyle(document.documentElement)
        .getPropertyValue(`--${variable}`)
        .trim()
    }
    return ''
  }

  // 设置主题CSS变量
  const setThemeVariable = (variable: string, value: string) => {
    if (typeof document !== 'undefined') {
      document.documentElement.style.setProperty(`--${variable}`, value)
    }
  }

  // 创建主题动画
  const createThemeTransition = () => {
    if (typeof document !== 'undefined') {
      const style = document.createElement('style')
      style.textContent = `
        * {
          transition: background-color 0.3s ease, 
                      color 0.3s ease, 
                      border-color 0.3s ease,
                      box-shadow 0.3s ease !important;
        }
      `
      document.head.appendChild(style)
      
      // 300ms后移除过渡效果，避免影响其他动画
      setTimeout(() => {
        document.head.removeChild(style)
      }, 300)
    }
  }

  // 主题预设
  const themePresets = {
    // 专业交易深色主题
    professional: {
      '--bg-primary': '#0a0e1a',
      '--bg-secondary': '#111827',
      '--accent-primary': '#1890ff',
      '--success-color': '#52c41a',
      '--error-color': '#ff4d4f'
    },
    
    // 经典深色主题
    classic: {
      '--bg-primary': '#1a1a1a',
      '--bg-secondary': '#2d2d2d',
      '--accent-primary': '#409eff',
      '--success-color': '#67c23a',
      '--error-color': '#f56c6c'
    },
    
    // 蓝色主题
    blue: {
      '--bg-primary': '#0f1419',
      '--bg-secondary': '#1e2328',
      '--accent-primary': '#3b82f6',
      '--success-color': '#10b981',
      '--error-color': '#ef4444'
    }
  }

  // 应用主题预设
  const applyThemePreset = (presetName: keyof typeof themePresets) => {
    const preset = themePresets[presetName]
    if (preset) {
      Object.entries(preset).forEach(([variable, value]) => {
        setThemeVariable(variable.replace('--', ''), value)
      })
    }
  }

  // 监听主题变化
  watch(appliedTheme, (newTheme: string) => {
    createThemeTransition()
    applyTheme(newTheme)
  }, { immediate: true })

  // 初始化
  onMounted(() => {
    detectSystemTheme()
    loadThemeFromStorage()
  })

  return {
    // 状态
    currentTheme,
    appliedTheme,
    systemTheme,
    
    // 计算属性
    isDark,
    isLight,
    isHighContrast,
    
    // 方法
    setTheme,
    toggleTheme,
    getThemeInfo,
    getAvailableThemes,
    getThemeVariable,
    setThemeVariable,
    applyThemePreset,
    
    // 工具
    createThemeTransition
  }
}

// 主题切换组合式函数
export const useThemeToggle = () => {
  const { currentTheme, setTheme, getAvailableThemes } = useTheme()
  
  const themes = getAvailableThemes()
  
  const nextTheme = computed(() => {
    const currentIndex = themes.findIndex(t => t.value === currentTheme.value)
    const nextIndex = (currentIndex + 1) % themes.length
    return themes[nextIndex]
  })
  
  const toggleToNext = () => {
    setTheme(nextTheme.value.value)
  }
  
  return {
    currentTheme,
    themes,
    nextTheme,
    toggleToNext,
    setTheme
  }
}

// 主题同步Hook（用于跨标签页同步）
export const useThemeSync = () => {
  const { setTheme } = useTheme()
  
  onMounted(() => {
    // 监听storage事件，实现跨标签页主题同步
    const handleStorageChange = (e: StorageEvent) => {
      if (e.key === 'trading-platform-theme' && e.newValue) {
        setTheme(e.newValue as ThemeMode)
      }
    }
    
    window.addEventListener('storage', handleStorageChange)
    
    // 清理
    return () => {
      window.removeEventListener('storage', handleStorageChange)
    }
  })
}

// 主题适配Hook（用于第三方组件主题适配）
export const useThemeAdapter = () => {
  const { appliedTheme, isDark } = useTheme()
  
  // Element Plus主题适配
  const getElementTheme = () => {
    return isDark.value ? 'dark' : 'light'
  }
  
  // Chart.js主题适配
  const getChartTheme = () => {
    return {
      backgroundColor: isDark.value ? '#111827' : '#ffffff',
      textColor: isDark.value ? '#f9fafb' : '#0f172a',
      gridColor: isDark.value ? '#374151' : '#e5e7eb'
    }
  }
  
  // TradingView主题适配
  const getTradingViewTheme = () => {
    return isDark.value ? 'dark' : 'light'
  }
  
  return {
    appliedTheme,
    isDark,
    getElementTheme,
    getChartTheme,
    getTradingViewTheme
  }
}