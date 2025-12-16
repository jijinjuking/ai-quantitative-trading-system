import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import axios from 'axios'

export interface User {
  id: string
  username: string
  email: string
  avatar?: string
  role: 'USER' | 'VIP' | 'ADMIN'
  status: 'ACTIVE' | 'SUSPENDED' | 'PENDING'
  createdAt: number
  lastLoginAt: number
  preferences: UserPreferences
  subscription: UserSubscription
  kyc: KYCInfo
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'auto'
  language: 'zh-CN' | 'en-US' | 'ja-JP'
  timezone: string
  currency: string
  notifications: {
    email: boolean
    push: boolean
    sms: boolean
    trading: boolean
    news: boolean
    system: boolean
  }
  trading: {
    confirmOrders: boolean
    soundEnabled: boolean
    defaultLeverage: number
    riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
    autoClose: boolean
  }
  display: {
    showBalance: boolean
    showPnL: boolean
    compactMode: boolean
    chartType: 'candlestick' | 'line' | 'area'
  }
}

export interface UserSubscription {
  plan: 'FREE' | 'BASIC' | 'PRO' | 'ENTERPRISE'
  status: 'ACTIVE' | 'EXPIRED' | 'CANCELED'
  startDate: number
  endDate: number
  features: string[]
  limits: {
    maxStrategies: number
    maxPositions: number
    apiCallsPerDay: number
    dataRetention: number
  }
}

export interface KYCInfo {
  status: 'NONE' | 'PENDING' | 'APPROVED' | 'REJECTED'
  level: 0 | 1 | 2 | 3
  documents: Array<{
    type: 'ID' | 'PASSPORT' | 'DRIVER_LICENSE' | 'PROOF_OF_ADDRESS'
    status: 'PENDING' | 'APPROVED' | 'REJECTED'
    uploadedAt: number
  }>
  limits: {
    dailyWithdraw: number
    monthlyWithdraw: number
    maxLeverage: number
  }
}

export interface WatchlistItem {
  symbol: string
  addedAt: number
  alerts: Array<{
    id: string
    type: 'PRICE_ABOVE' | 'PRICE_BELOW' | 'VOLUME_SPIKE' | 'CUSTOM'
    value: number
    enabled: boolean
    triggered: boolean
  }>
}

export interface TradingStrategy {
  id: string
  name: string
  description: string
  type: 'AI' | 'MANUAL' | 'COPY'
  status: 'ACTIVE' | 'INACTIVE' | 'PAUSED'
  symbols: string[]
  parameters: Record<string, any>
  performance: {
    totalReturn: number
    winRate: number
    maxDrawdown: number
    sharpeRatio: number
  }
  createdAt: number
  updatedAt: number
}

export interface UserStats {
  totalTrades: number
  totalVolume: number
  totalPnL: number
  winRate: number
  avgHoldTime: number
  bestTrade: number
  worstTrade: number
  tradingDays: number
  favoriteSymbols: string[]
  tradingHours: Record<string, number>
}

export const useUserStore = defineStore('user', () => {
  // 状态
  const user = ref<User | null>(null)
  const watchlist = ref<WatchlistItem[]>([])
  const strategies = ref<TradingStrategy[]>([])
  const userStats = ref<UserStats | null>(null)
  const isAuthenticated = ref(false)
  const isLoading = ref(false)
  const lastError = ref<string | null>(null)

  // 计算属性
  const userRole = computed(() => user.value?.role || 'USER')
  
  const isVIP = computed(() => user.value?.role === 'VIP' || user.value?.role === 'ADMIN')
  
  const isAdmin = computed(() => user.value?.role === 'ADMIN')
  
  const subscriptionPlan = computed(() => user.value?.subscription.plan || 'FREE')
  
  const kycLevel = computed(() => user.value?.kyc.level || 0)
  
  const canTrade = computed(() => {
    return isAuthenticated.value && 
           user.value?.status === 'ACTIVE' && 
           user.value?.kyc.level >= 1
  })
  
  const watchlistSymbols = computed(() => {
    return watchlist.value.map(item => item.symbol)
  })
  
  const activeStrategies = computed(() => {
    return strategies.value.filter(s => s.status === 'ACTIVE')
  })

  // 登录
  const login = async (credentials: { email: string; password: string }): Promise<void> => {
    try {
      isLoading.value = true
      lastError.value = null

      const response = await axios.post('/api/auth/login', credentials)
      const { user: userData, token } = response.data

      // 保存token
      localStorage.setItem('auth_token', token)
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`

      user.value = userData
      isAuthenticated.value = true

      // 加载用户相关数据
      await loadUserData()
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '登录失败'
      console.error('Login failed:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  // 注册
  const register = async (userData: {
    username: string
    email: string
    password: string
    confirmPassword: string
  }): Promise<void> => {
    try {
      isLoading.value = true
      lastError.value = null

      const response = await axios.post('/api/auth/register', userData)
      const { user: newUser, token } = response.data

      // 保存token
      localStorage.setItem('auth_token', token)
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`

      user.value = newUser
      isAuthenticated.value = true

      await loadUserData()
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '注册失败'
      console.error('Registration failed:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  // 登出
  const logout = async (): Promise<void> => {
    try {
      await axios.post('/api/auth/logout')
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      // 清除本地数据
      user.value = null
      watchlist.value = []
      strategies.value = []
      userStats.value = null
      isAuthenticated.value = false
      
      localStorage.removeItem('auth_token')
      delete axios.defaults.headers.common['Authorization']
    }
  }

  // 检查认证状态
  const checkAuth = async (): Promise<void> => {
    const token = localStorage.getItem('auth_token')
    if (!token) return

    try {
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`
      const response = await axios.get('/api/auth/me')
      
      user.value = response.data
      isAuthenticated.value = true
      
      await loadUserData()
    } catch (error) {
      console.error('Auth check failed:', error)
      await logout()
    }
  }

  // 加载用户数据
  const loadUserData = async (): Promise<void> => {
    if (!isAuthenticated.value) return

    try {
      const [watchlistRes, strategiesRes, statsRes] = await Promise.all([
        axios.get('/api/user/watchlist'),
        axios.get('/api/user/strategies'),
        axios.get('/api/user/stats')
      ])

      watchlist.value = watchlistRes.data
      strategies.value = strategiesRes.data
      userStats.value = statsRes.data
    } catch (error) {
      console.error('Failed to load user data:', error)
    }
  }

  // 更新用户信息
  const updateProfile = async (updates: Partial<User>): Promise<void> => {
    try {
      isLoading.value = true
      
      const response = await axios.put('/api/user/profile', updates)
      user.value = response.data
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '更新用户信息失败'
      console.error('Failed to update profile:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  // 更新偏好设置
  const updatePreferences = async (preferences: Partial<UserPreferences>): Promise<void> => {
    try {
      const response = await axios.put('/api/user/preferences', preferences)
      
      if (user.value) {
        user.value.preferences = { ...user.value.preferences, ...preferences }
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '更新偏好设置失败'
      console.error('Failed to update preferences:', error)
      throw error
    }
  }

  // 添加到自选
  const addToWatchlist = async (symbol: string): Promise<void> => {
    try {
      await axios.post('/api/user/watchlist', { symbol })
      
      const newItem: WatchlistItem = {
        symbol,
        addedAt: Date.now(),
        alerts: []
      }
      
      watchlist.value.push(newItem)
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '添加自选失败'
      console.error('Failed to add to watchlist:', error)
      throw error
    }
  }

  // 从自选移除
  const removeFromWatchlist = async (symbol: string): Promise<void> => {
    try {
      await axios.delete(`/api/user/watchlist/${symbol}`)
      
      const index = watchlist.value.findIndex(item => item.symbol === symbol)
      if (index !== -1) {
        watchlist.value.splice(index, 1)
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '移除自选失败'
      console.error('Failed to remove from watchlist:', error)
      throw error
    }
  }

  // 切换自选状态
  const toggleWatchlist = async (symbol: string): Promise<void> => {
    const isInWatchlist = watchlist.value.some(item => item.symbol === symbol)
    
    if (isInWatchlist) {
      await removeFromWatchlist(symbol)
    } else {
      await addToWatchlist(symbol)
    }
  }

  // 设置价格提醒
  const setAlert = async (symbol: string, alert: {
    type: 'PRICE_ABOVE' | 'PRICE_BELOW' | 'VOLUME_SPIKE' | 'CUSTOM'
    value: number
  }): Promise<void> => {
    try {
      const response = await axios.post(`/api/user/alerts`, { symbol, ...alert })
      const newAlert = response.data
      
      const watchlistItem = watchlist.value.find(item => item.symbol === symbol)
      if (watchlistItem) {
        watchlistItem.alerts.push(newAlert)
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '设置提醒失败'
      console.error('Failed to set alert:', error)
      throw error
    }
  }

  // 删除提醒
  const removeAlert = async (alertId: string): Promise<void> => {
    try {
      await axios.delete(`/api/user/alerts/${alertId}`)
      
      // 从本地数据中移除
      watchlist.value.forEach(item => {
        const index = item.alerts.findIndex(alert => alert.id === alertId)
        if (index !== -1) {
          item.alerts.splice(index, 1)
        }
      })
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '删除提醒失败'
      console.error('Failed to remove alert:', error)
      throw error
    }
  }

  // 上传头像
  const uploadAvatar = async (file: File): Promise<void> => {
    try {
      isLoading.value = true
      
      const formData = new FormData()
      formData.append('avatar', file)
      
      const response = await axios.post('/api/user/avatar', formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      })
      
      if (user.value) {
        user.value.avatar = response.data.avatarUrl
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '上传头像失败'
      console.error('Failed to upload avatar:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  // 修改密码
  const changePassword = async (passwords: {
    currentPassword: string
    newPassword: string
    confirmPassword: string
  }): Promise<void> => {
    try {
      isLoading.value = true
      
      await axios.put('/api/user/password', passwords)
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '修改密码失败'
      console.error('Failed to change password:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  // KYC认证
  const submitKYC = async (documents: File[]): Promise<void> => {
    try {
      isLoading.value = true
      
      const formData = new FormData()
      documents.forEach((file, index) => {
        formData.append(`document_${index}`, file)
      })
      
      const response = await axios.post('/api/user/kyc', formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      })
      
      if (user.value) {
        user.value.kyc = response.data
      }
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : 'KYC提交失败'
      console.error('Failed to submit KYC:', error)
      throw error
    } finally {
      isLoading.value = false
    }
  }

  // 获取用户统计
  const refreshStats = async (): Promise<void> => {
    try {
      const response = await axios.get('/api/user/stats')
      userStats.value = response.data
    } catch (error) {
      console.error('Failed to refresh stats:', error)
    }
  }

  return {
    // 状态
    user: readonly(user),
    watchlist: readonly(watchlist),
    strategies: readonly(strategies),
    userStats: readonly(userStats),
    isAuthenticated: readonly(isAuthenticated),
    isLoading: readonly(isLoading),
    lastError: readonly(lastError),
    
    // 计算属性
    userRole,
    isVIP,
    isAdmin,
    subscriptionPlan,
    kycLevel,
    canTrade,
    watchlistSymbols,
    activeStrategies,
    
    // 方法
    login,
    register,
    logout,
    checkAuth,
    loadUserData,
    updateProfile,
    updatePreferences,
    addToWatchlist,
    removeFromWatchlist,
    toggleWatchlist,
    setAlert,
    removeAlert,
    uploadAvatar,
    changePassword,
    submitKYC,
    refreshStats
  }
})