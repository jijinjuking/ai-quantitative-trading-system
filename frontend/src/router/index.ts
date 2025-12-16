import { createRouter, createWebHistory } from 'vue-router'
import TradingDashboard from '@/views/TradingDashboard.vue'
import TestPage from '@/views/TestPage.vue'
import SimpleTest from '@/views/SimpleTest.vue'

const routes = [
  {
    path: '/',
    name: 'SimpleTest',
    component: SimpleTest
  },
  {
    path: '/trading',
    name: 'Trading',
    component: TradingDashboard
  },
  {
    path: '/test',
    name: 'TestPage',
    component: TestPage
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router