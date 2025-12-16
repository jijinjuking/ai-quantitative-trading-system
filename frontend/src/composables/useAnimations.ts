import { ref, nextTick, onMounted, onUnmounted } from 'vue'
import { gsap } from 'gsap'

// 动画配置类型
interface AnimationConfig {
  duration?: number
  delay?: number
  ease?: string
  repeat?: number
  yoyo?: boolean
}

// 价格闪烁动画配置
interface PriceFlashConfig extends AnimationConfig {
  color?: string
  intensity?: number
}

// 数字滚动动画配置
interface CountUpConfig extends AnimationConfig {
  startValue?: number
  endValue: number
  decimals?: number
  separator?: string
}

export const useAnimations = () => {
  // 动画实例存储
  const animations = ref<Map<string, gsap.core.Timeline>>(new Map())
  
  // 默认动画配置
  const defaultConfig: AnimationConfig = {
    duration: 0.3,
    delay: 0,
    ease: 'power2.out',
    repeat: 0,
    yoyo: false
  }

  // 创建时间线动画
  const createTimeline = (id: string): gsap.core.Timeline => {
    const tl = gsap.timeline()
    animations.value.set(id, tl)
    return tl
  }

  // 获取动画实例
  const getAnimation = (id: string): gsap.core.Timeline | undefined => {
    return animations.value.get(id)
  }

  // 停止动画
  const stopAnimation = (id: string) => {
    const animation = animations.value.get(id)
    if (animation) {
      animation.kill()
      animations.value.delete(id)
    }
  }

  // 停止所有动画
  const stopAllAnimations = () => {
    animations.value.forEach((animation) => {
      animation.kill()
    })
    animations.value.clear()
  }

  // ==================== 基础动画 ====================

  // 淡入动画
  const fadeIn = (
    element: HTMLElement | string, 
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, ...config }
    
    return new Promise((resolve) => {
      gsap.fromTo(element, 
        { opacity: 0 },
        {
          opacity: 1,
          duration: finalConfig.duration,
          delay: finalConfig.delay,
          ease: finalConfig.ease,
          onComplete: resolve
        }
      )
    })
  }

  // 淡出动画
  const fadeOut = (
    element: HTMLElement | string, 
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, ...config }
    
    return new Promise((resolve) => {
      gsap.to(element, {
        opacity: 0,
        duration: finalConfig.duration,
        delay: finalConfig.delay,
        ease: finalConfig.ease,
        onComplete: resolve
      })
    })
  }

  // 滑入动画
  const slideIn = (
    element: HTMLElement | string,
    direction: 'up' | 'down' | 'left' | 'right' = 'up',
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, ...config }
    
    const fromProps: any = { opacity: 0 }
    const toProps: any = { opacity: 1 }
    
    switch (direction) {
      case 'up':
        fromProps.y = 30
        toProps.y = 0
        break
      case 'down':
        fromProps.y = -30
        toProps.y = 0
        break
      case 'left':
        fromProps.x = 30
        toProps.x = 0
        break
      case 'right':
        fromProps.x = -30
        toProps.x = 0
        break
    }
    
    return new Promise((resolve) => {
      gsap.fromTo(element, fromProps, {
        ...toProps,
        duration: finalConfig.duration,
        delay: finalConfig.delay,
        ease: finalConfig.ease,
        onComplete: resolve
      })
    })
  }

  // 缩放动画
  const scaleIn = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, ...config }
    
    return new Promise((resolve) => {
      gsap.fromTo(element,
        { scale: 0.8, opacity: 0 },
        {
          scale: 1,
          opacity: 1,
          duration: finalConfig.duration,
          delay: finalConfig.delay,
          ease: finalConfig.ease,
          onComplete: resolve
        }
      )
    })
  }

  // 弹跳动画
  const bounce = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, duration: 0.6, ease: 'bounce.out', ...config }
    
    return new Promise((resolve) => {
      gsap.fromTo(element,
        { y: -20 },
        {
          y: 0,
          duration: finalConfig.duration,
          delay: finalConfig.delay,
          ease: finalConfig.ease,
          onComplete: resolve
        }
      )
    })
  }

  // 摇摆动画
  const shake = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, duration: 0.5, ...config }
    
    return new Promise((resolve) => {
      gsap.to(element, {
        x: [-10, 10, -8, 8, -6, 6, -4, 4, -2, 2, 0],
        duration: finalConfig.duration,
        delay: finalConfig.delay,
        ease: 'power2.out',
        onComplete: resolve
      })
    })
  }

  // ==================== 专业交易动画 ====================

  // 价格闪烁动画
  const priceFlash = (
    element: HTMLElement | string,
    type: 'up' | 'down' = 'up',
    config: PriceFlashConfig = {}
  ): Promise<void> => {
    const finalConfig = { 
      duration: 0.6, 
      intensity: 0.3,
      color: type === 'up' ? '#52c41a' : '#ff4d4f',
      ...config 
    }
    
    return new Promise((resolve) => {
      const tl = gsap.timeline({ onComplete: resolve })
      
      tl.to(element, {
        backgroundColor: finalConfig.color,
        duration: 0.1,
        ease: 'power2.out'
      })
      .to(element, {
        backgroundColor: 'transparent',
        duration: finalConfig.duration! - 0.1,
        ease: 'power2.out'
      })
    })
  }

  // 数字跳动动画
  const numberBounce = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, duration: 0.4, ...config }
    
    return new Promise((resolve) => {
      gsap.to(element, {
        y: [-3, 0],
        scale: [1.05, 1],
        duration: finalConfig.duration,
        delay: finalConfig.delay,
        ease: 'back.out(1.7)',
        onComplete: resolve
      })
    })
  }

  // 数字滚动动画
  const countUp = (
    element: HTMLElement | string,
    config: CountUpConfig
  ): Promise<void> => {
    const finalConfig = {
      startValue: 0,
      decimals: 2,
      separator: ',',
      duration: 1,
      ...config
    }
    
    return new Promise((resolve) => {
      const obj = { value: finalConfig.startValue }
      
      gsap.to(obj, {
        value: finalConfig.endValue,
        duration: finalConfig.duration,
        ease: 'power2.out',
        onUpdate: () => {
          const currentValue = obj.value.toFixed(finalConfig.decimals)
          const formattedValue = finalConfig.separator 
            ? currentValue.replace(/\B(?=(\d{3})+(?!\d))/g, finalConfig.separator)
            : currentValue
          
          if (typeof element === 'string') {
            const el = document.querySelector(element)
            if (el) el.textContent = formattedValue
          } else {
            element.textContent = formattedValue
          }
        },
        onComplete: resolve
      })
    })
  }

  // 进度条动画
  const progressBar = (
    element: HTMLElement | string,
    percentage: number,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, duration: 1, ...config }
    
    return new Promise((resolve) => {
      gsap.fromTo(element,
        { width: '0%' },
        {
          width: `${percentage}%`,
          duration: finalConfig.duration,
          delay: finalConfig.delay,
          ease: finalConfig.ease,
          onComplete: resolve
        }
      )
    })
  }

  // 脉冲动画
  const pulse = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): gsap.core.Timeline => {
    const finalConfig = { 
      duration: 1, 
      repeat: -1, 
      yoyo: true, 
      ease: 'power2.inOut',
      ...config 
    }
    
    const tl = gsap.timeline({ repeat: finalConfig.repeat })
    
    tl.to(element, {
      scale: 1.05,
      opacity: 0.8,
      duration: finalConfig.duration,
      ease: finalConfig.ease,
      yoyo: finalConfig.yoyo
    })
    
    return tl
  }

  // 发光效果动画
  const glow = (
    element: HTMLElement | string,
    color: string = '#1890ff',
    config: AnimationConfig = {}
  ): gsap.core.Timeline => {
    const finalConfig = { 
      duration: 1, 
      repeat: -1, 
      yoyo: true,
      ...config 
    }
    
    const tl = gsap.timeline({ repeat: finalConfig.repeat })
    
    tl.to(element, {
      boxShadow: `0 0 20px ${color}`,
      duration: finalConfig.duration,
      ease: 'power2.inOut',
      yoyo: finalConfig.yoyo
    })
    
    return tl
  }

  // ==================== 列表动画 ====================

  // 交错动画
  const staggerIn = (
    elements: HTMLElement[] | string,
    config: AnimationConfig & { stagger?: number } = {}
  ): Promise<void> => {
    const finalConfig = { 
      ...defaultConfig, 
      stagger: 0.1,
      ...config 
    }
    
    return new Promise((resolve) => {
      gsap.fromTo(elements,
        { y: 30, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration: finalConfig.duration,
          delay: finalConfig.delay,
          ease: finalConfig.ease,
          stagger: finalConfig.stagger,
          onComplete: resolve
        }
      )
    })
  }

  // 列表项添加动画
  const listItemAdd = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, duration: 0.4, ...config }
    
    return new Promise((resolve) => {
      gsap.fromTo(element,
        { 
          height: 0, 
          opacity: 0, 
          scale: 0.8,
          transformOrigin: 'top center'
        },
        {
          height: 'auto',
          opacity: 1,
          scale: 1,
          duration: finalConfig.duration,
          delay: finalConfig.delay,
          ease: 'back.out(1.7)',
          onComplete: resolve
        }
      )
    })
  }

  // 列表项删除动画
  const listItemRemove = (
    element: HTMLElement | string,
    config: AnimationConfig = {}
  ): Promise<void> => {
    const finalConfig = { ...defaultConfig, duration: 0.3, ...config }
    
    return new Promise((resolve) => {
      gsap.to(element, {
        height: 0,
        opacity: 0,
        scale: 0.8,
        duration: finalConfig.duration,
        delay: finalConfig.delay,
        ease: 'power2.in',
        onComplete: resolve
      })
    })
  }

  // ==================== 页面转场动画 ====================

  // 页面淡入转场
  const pageTransitionFade = (
    enterElement: HTMLElement | string,
    leaveElement?: HTMLElement | string
  ): Promise<void> => {
    return new Promise((resolve) => {
      const tl = gsap.timeline({ onComplete: resolve })
      
      if (leaveElement) {
        tl.to(leaveElement, {
          opacity: 0,
          duration: 0.2,
          ease: 'power2.out'
        })
      }
      
      tl.fromTo(enterElement,
        { opacity: 0 },
        {
          opacity: 1,
          duration: 0.3,
          ease: 'power2.out'
        }
      )
    })
  }

  // 页面滑动转场
  const pageTransitionSlide = (
    enterElement: HTMLElement | string,
    leaveElement?: HTMLElement | string,
    direction: 'left' | 'right' = 'left'
  ): Promise<void> => {
    return new Promise((resolve) => {
      const tl = gsap.timeline({ onComplete: resolve })
      
      const enterX = direction === 'left' ? '100%' : '-100%'
      const leaveX = direction === 'left' ? '-100%' : '100%'
      
      if (leaveElement) {
        tl.to(leaveElement, {
          x: leaveX,
          duration: 0.3,
          ease: 'power2.inOut'
        })
      }
      
      tl.fromTo(enterElement,
        { x: enterX },
        {
          x: '0%',
          duration: 0.3,
          ease: 'power2.inOut'
        },
        leaveElement ? '-=0.15' : 0
      )
    })
  }

  // ==================== 工具函数 ====================

  // 等待下一帧
  const waitForNextFrame = (): Promise<void> => {
    return new Promise((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => resolve())
      })
    })
  }

  // 等待指定时间
  const wait = (ms: number): Promise<void> => {
    return new Promise((resolve) => {
      setTimeout(resolve, ms)
    })
  }

  // 检查元素是否在视口中
  const isInViewport = (element: HTMLElement): boolean => {
    const rect = element.getBoundingClientRect()
    return (
      rect.top >= 0 &&
      rect.left >= 0 &&
      rect.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
      rect.right <= (window.innerWidth || document.documentElement.clientWidth)
    )
  }

  // 滚动到元素
  const scrollToElement = (
    element: HTMLElement | string,
    config: AnimationConfig & { offset?: number } = {}
  ): Promise<void> => {
    const finalConfig = { 
      duration: 1, 
      offset: 0, 
      ease: 'power2.out',
      ...config 
    }
    
    return new Promise((resolve) => {
      gsap.to(window, {
        scrollTo: {
          y: element,
          offsetY: finalConfig.offset
        },
        duration: finalConfig.duration,
        ease: finalConfig.ease,
        onComplete: resolve
      })
    })
  }

  // 清理函数
  onUnmounted(() => {
    stopAllAnimations()
  })

  return {
    // 基础动画
    fadeIn,
    fadeOut,
    slideIn,
    scaleIn,
    bounce,
    shake,
    
    // 专业交易动画
    priceFlash,
    numberBounce,
    countUp,
    progressBar,
    pulse,
    glow,
    
    // 列表动画
    staggerIn,
    listItemAdd,
    listItemRemove,
    
    // 页面转场
    pageTransitionFade,
    pageTransitionSlide,
    
    // 动画管理
    createTimeline,
    getAnimation,
    stopAnimation,
    stopAllAnimations,
    
    // 工具函数
    waitForNextFrame,
    wait,
    isInViewport,
    scrollToElement
  }
}

// 价格动画Hook
export const usePriceAnimations = () => {
  const { priceFlash, numberBounce } = useAnimations()
  
  // 价格变化动画
  const animatePriceChange = (
    element: HTMLElement | string,
    oldPrice: number,
    newPrice: number
  ) => {
    if (newPrice > oldPrice) {
      priceFlash(element, 'up')
      numberBounce(element)
    } else if (newPrice < oldPrice) {
      priceFlash(element, 'down')
      numberBounce(element)
    }
  }
  
  return {
    animatePriceChange,
    priceFlash,
    numberBounce
  }
}

// 列表动画Hook
export const useListAnimations = () => {
  const { staggerIn, listItemAdd, listItemRemove } = useAnimations()
  
  // 列表初始化动画
  const initializeList = (selector: string) => {
    nextTick(() => {
      const items = document.querySelectorAll(selector)
      if (items.length > 0) {
        staggerIn(Array.from(items) as HTMLElement[])
      }
    })
  }
  
  return {
    initializeList,
    staggerIn,
    listItemAdd,
    listItemRemove
  }
}