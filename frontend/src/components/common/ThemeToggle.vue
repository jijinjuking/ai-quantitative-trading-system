<template>
  <div class="theme-toggle">
    <!-- 简单切换按钮 -->
    <el-button 
      v-if="mode === 'simple'"
      :icon="currentThemeIcon"
      circle
      @click="toggleToNext"
      class="theme-toggle-btn"
      :title="nextTheme.name"
    />
    
    <!-- 下拉选择器 -->
    <el-dropdown 
      v-else-if="mode === 'dropdown'"
      @command="handleThemeSelect"
      placement="bottom-end"
    >
      <el-button class="theme-toggle-btn">
        <i :class="currentThemeIcon"></i>
        <span class="theme-name">{{ currentThemeInfo.name }}</span>
        <i class="el-icon-arrow-down"></i>
      </el-button>
      
      <template #dropdown>
        <el-dropdown-menu class="theme-dropdown-menu">
          <el-dropdown-item
            v-for="theme in themes"
            :key="theme.value"
            :command="theme.value"
            :class="{ 'is-active': currentTheme === theme.value }"
          >
            <div class="theme-option">
              <i :class="theme.icon" class="theme-icon"></i>
              <div class="theme-info">
                <div class="theme-name">{{ theme.name }}</div>
                <div class="theme-description">{{ theme.description }}</div>
              </div>
              <i 
                v-if="currentTheme === theme.value"
                class="el-icon-check theme-check"
              ></i>
            </div>
          </el-dropdown-item>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
    
    <!-- 面板模式 -->
    <div v-else-if="mode === 'panel'" class="theme-panel">
      <div class="panel-header">
        <h4>主题设置</h4>
        <el-button 
          text 
          @click="$emit('close')"
          class="close-btn"
        >
          <i class="el-icon-close"></i>
        </el-button>
      </div>
      
      <div class="panel-content">
        <!-- 主题选项 -->
        <div class="theme-options">
          <div
            v-for="theme in themes"
            :key="theme.value"
            class="theme-card"
            :class="{ 'is-active': currentTheme === theme.value }"
            @click="setTheme(theme.value)"
          >
            <div class="theme-preview">
              <div class="preview-bg" :data-theme="theme.value">
                <div class="preview-header"></div>
                <div class="preview-content">
                  <div class="preview-sidebar"></div>
                  <div class="preview-main">
                    <div class="preview-chart"></div>
                    <div class="preview-data"></div>
                  </div>
                </div>
              </div>
            </div>
            
            <div class="theme-meta">
              <div class="theme-header">
                <i :class="theme.icon" class="theme-icon"></i>
                <span class="theme-name">{{ theme.name }}</span>
                <i 
                  v-if="currentTheme === theme.value"
                  class="el-icon-check theme-check"
                ></i>
              </div>
              <p class="theme-description">{{ theme.description }}</p>
            </div>
          </div>
        </div>
        
        <!-- 自定义设置 -->
        <div class="custom-settings" v-if="showCustomSettings">
          <h5>自定义设置</h5>
          
          <div class="setting-item">
            <label>主色调</label>
            <el-color-picker 
              v-model="customPrimaryColor"
              @change="onPrimaryColorChange"
            />
          </div>
          
          <div class="setting-item">
            <label>成功色</label>
            <el-color-picker 
              v-model="customSuccessColor"
              @change="onSuccessColorChange"
            />
          </div>
          
          <div class="setting-item">
            <label>错误色</label>
            <el-color-picker 
              v-model="customErrorColor"
              @change="onErrorColorChange"
            />
          </div>
          
          <div class="setting-actions">
            <el-button size="small" @click="resetCustomColors">
              重置
            </el-button>
            <el-button size="small" type="primary" @click="saveCustomTheme">
              保存
            </el-button>
          </div>
        </div>
        
        <!-- 预设主题 -->
        <div class="preset-themes" v-if="showPresets">
          <h5>预设主题</h5>
          
          <div class="preset-grid">
            <div
              v-for="(preset, name) in themePresets"
              :key="name"
              class="preset-item"
              @click="applyThemePreset(name)"
            >
              <div class="preset-preview" :style="getPresetStyle(preset)"></div>
              <span class="preset-name">{{ getPresetName(name) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTheme, useThemeToggle, type ThemeMode } from '@/composables/useTheme'
import { useAnimations } from '@/composables/useAnimations'

// Props
interface Props {
  mode?: 'simple' | 'dropdown' | 'panel'
  showCustomSettings?: boolean
  showPresets?: boolean
  animated?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  mode: 'simple',
  showCustomSettings: false,
  showPresets: false,
  animated: true
})

// Emits
const emit = defineEmits<{
  close: []
  themeChange: [theme: ThemeMode]
}>()

// Composables
const { 
  currentTheme, 
  setTheme: setThemeBase, 
  getThemeInfo, 
  getThemeVariable,
  setThemeVariable,
  applyThemePreset
} = useTheme()

const { 
  themes, 
  nextTheme, 
  toggleToNext: toggleToNextBase 
} = useThemeToggle()

const { fadeIn, scaleIn } = useAnimations()

// 响应式数据
const customPrimaryColor = ref('#1890ff')
const customSuccessColor = ref('#52c41a')
const customErrorColor = ref('#ff4d4f')

// 主题预设
const themePresets = {
  professional: {
    '--accent-primary': '#1890ff',
    '--success-color': '#52c41a',
    '--error-color': '#ff4d4f'
  },
  classic: {
    '--accent-primary': '#409eff',
    '--success-color': '#67c23a',
    '--error-color': '#f56c6c'
  },
  blue: {
    '--accent-primary': '#3b82f6',
    '--success-color': '#10b981',
    '--error-color': '#ef4444'
  },
  purple: {
    '--accent-primary': '#8b5cf6',
    '--success-color': '#06d6a0',
    '--error-color': '#f72585'
  },
  green: {
    '--accent-primary': '#059669',
    '--success-color': '#10b981',
    '--error-color': '#dc2626'
  }
}

// 计算属性
const currentThemeInfo = computed(() => {
  return getThemeInfo(currentTheme.value)
})

const currentThemeIcon = computed(() => {
  return currentThemeInfo.value.icon
})

// 方法
const setTheme = (theme: ThemeMode) => {
  if (props.animated) {
    // 添加切换动画
    const body = document.body
    fadeIn(body, { duration: 0.3 })
  }
  
  setThemeBase(theme)
  emit('themeChange', theme)
}

const toggleToNext = () => {
  if (props.animated) {
    const toggleBtn = document.querySelector('.theme-toggle-btn')
    if (toggleBtn) {
      scaleIn(toggleBtn as HTMLElement, { duration: 0.2 })
    }
  }
  
  toggleToNextBase()
  emit('themeChange', currentTheme.value)
}

const handleThemeSelect = (theme: ThemeMode) => {
  setTheme(theme)
}

// 自定义颜色处理
const onPrimaryColorChange = (color: string) => {
  setThemeVariable('accent-primary', color)
}

const onSuccessColorChange = (color: string) => {
  setThemeVariable('success-color', color)
}

const onErrorColorChange = (color: string) => {
  setThemeVariable('error-color', color)
}

const resetCustomColors = () => {
  customPrimaryColor.value = '#1890ff'
  customSuccessColor.value = '#52c41a'
  customErrorColor.value = '#ff4d4f'
  
  onPrimaryColorChange(customPrimaryColor.value)
  onSuccessColorChange(customSuccessColor.value)
  onErrorColorChange(customErrorColor.value)
}

const saveCustomTheme = () => {
  const customTheme = {
    '--accent-primary': customPrimaryColor.value,
    '--success-color': customSuccessColor.value,
    '--error-color': customErrorColor.value
  }
  
  localStorage.setItem('custom-theme', JSON.stringify(customTheme))
  // ElMessage.success('自定义主题已保存')
}

// 预设主题处理
const getPresetName = (name: string): string => {
  const names = {
    professional: '专业版',
    classic: '经典版',
    blue: '蓝色版',
    purple: '紫色版',
    green: '绿色版'
  }
  return names[name] || name
}

const getPresetStyle = (preset: any) => {
  return {
    background: `linear-gradient(135deg, ${preset['--accent-primary']}, ${preset['--success-color']})`
  }
}

// 初始化自定义颜色
const initializeCustomColors = () => {
  customPrimaryColor.value = getThemeVariable('accent-primary') || '#1890ff'
  customSuccessColor.value = getThemeVariable('success-color') || '#52c41a'
  customErrorColor.value = getThemeVariable('error-color') || '#ff4d4f'
}

// 监听主题变化
watch(currentTheme, () => {
  initializeCustomColors()
})

// 初始化
initializeCustomColors()
</script>

<style lang="scss" scoped>
.theme-toggle {
  .theme-toggle-btn {
    border-radius: var(--radius-full);
    transition: all var(--transition-normal) ease;
    
    &:hover {
      transform: scale(1.05);
      box-shadow: var(--shadow-md);
    }
    
    .theme-name {
      margin: 0 8px;
      font-size: var(--font-size-sm);
    }
  }
}

.theme-dropdown-menu {
  min-width: 280px;
  
  .theme-option {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    
    .theme-icon {
      font-size: 16px;
      color: var(--accent-primary);
    }
    
    .theme-info {
      flex: 1;
      
      .theme-name {
        font-weight: var(--font-weight-medium);
        color: var(--text-primary);
        margin-bottom: 2px;
      }
      
      .theme-description {
        font-size: var(--font-size-xs);
        color: var(--text-secondary);
        line-height: 1.3;
      }
    }
    
    .theme-check {
      color: var(--success-color);
      font-size: 14px;
    }
  }
  
  .el-dropdown-menu__item.is-active {
    background: var(--accent-primary-bg);
    color: var(--accent-primary);
  }
}

.theme-panel {
  width: 320px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--border-color);
    
    h4 {
      margin: 0;
      font-size: var(--font-size-lg);
      font-weight: var(--font-weight-semibold);
      color: var(--text-primary);
    }
    
    .close-btn {
      padding: 4px;
      border-radius: var(--radius-sm);
    }
  }
  
  .panel-content {
    padding: 16px;
    max-height: 500px;
    overflow-y: auto;
    @include custom-scrollbar;
  }
}

.theme-options {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
  margin-bottom: 24px;
  
  .theme-card {
    border: 2px solid var(--border-color);
    border-radius: var(--radius-lg);
    overflow: hidden;
    cursor: pointer;
    transition: all var(--transition-normal) ease;
    
    &:hover {
      border-color: var(--accent-primary);
      transform: translateY(-2px);
      box-shadow: var(--shadow-md);
    }
    
    &.is-active {
      border-color: var(--accent-primary);
      box-shadow: 0 0 0 1px var(--accent-primary-bg);
    }
    
    .theme-preview {
      height: 80px;
      position: relative;
      overflow: hidden;
      
      .preview-bg {
        width: 100%;
        height: 100%;
        position: relative;
        
        &[data-theme="light"] {
          background: #ffffff;
          
          .preview-header {
            background: #f8fafc;
            border-bottom: 1px solid #e2e8f0;
          }
          
          .preview-sidebar {
            background: #f1f5f9;
          }
          
          .preview-chart {
            background: linear-gradient(135deg, #3b82f6, #10b981);
          }
          
          .preview-data {
            background: #e2e8f0;
          }
        }
        
        &[data-theme="dark"] {
          background: #0a0e1a;
          
          .preview-header {
            background: #111827;
            border-bottom: 1px solid #374151;
          }
          
          .preview-sidebar {
            background: #1f2937;
          }
          
          .preview-chart {
            background: linear-gradient(135deg, #1890ff, #52c41a);
          }
          
          .preview-data {
            background: #374151;
          }
        }
        
        &[data-theme="high-contrast"] {
          background: #000000;
          
          .preview-header {
            background: #1a1a1a;
            border-bottom: 1px solid #666666;
          }
          
          .preview-sidebar {
            background: #333333;
          }
          
          .preview-chart {
            background: linear-gradient(135deg, #0099ff, #00ff00);
          }
          
          .preview-data {
            background: #666666;
          }
        }
      }
      
      .preview-header {
        height: 16px;
        width: 100%;
      }
      
      .preview-content {
        display: flex;
        height: calc(100% - 16px);
        
        .preview-sidebar {
          width: 30%;
        }
        
        .preview-main {
          flex: 1;
          display: flex;
          flex-direction: column;
          gap: 4px;
          padding: 4px;
          
          .preview-chart {
            flex: 1;
            border-radius: 2px;
          }
          
          .preview-data {
            height: 20px;
            border-radius: 2px;
          }
        }
      }
    }
    
    .theme-meta {
      padding: 12px;
      
      .theme-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 4px;
        
        .theme-icon {
          color: var(--accent-primary);
        }
        
        .theme-name {
          flex: 1;
          font-weight: var(--font-weight-medium);
          color: var(--text-primary);
        }
        
        .theme-check {
          color: var(--success-color);
        }
      }
      
      .theme-description {
        font-size: var(--font-size-xs);
        color: var(--text-secondary);
        line-height: 1.4;
        margin: 0;
      }
    }
  }
}

.custom-settings {
  margin-bottom: 24px;
  
  h5 {
    margin: 0 0 16px 0;
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }
  
  .setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
    
    label {
      font-size: var(--font-size-sm);
      color: var(--text-secondary);
    }
  }
  
  .setting-actions {
    display: flex;
    gap: 8px;
    margin-top: 16px;
  }
}

.preset-themes {
  h5 {
    margin: 0 0 16px 0;
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--text-primary);
  }
  
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    
    .preset-item {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 8px;
      cursor: pointer;
      padding: 8px;
      border-radius: var(--radius-md);
      transition: all var(--transition-normal) ease;
      
      &:hover {
        background: var(--bg-tertiary);
      }
      
      .preset-preview {
        width: 40px;
        height: 40px;
        border-radius: var(--radius-md);
        border: 2px solid var(--border-color);
      }
      
      .preset-name {
        font-size: var(--font-size-xs);
        color: var(--text-secondary);
        text-align: center;
      }
    }
  }
}

// 响应式设计
@media (max-width: 768px) {
  .theme-panel {
    width: 100%;
    max-width: 300px;
  }
  
  .preset-grid {
    grid-template-columns: repeat(2, 1fr) !important;
  }
}
</style>