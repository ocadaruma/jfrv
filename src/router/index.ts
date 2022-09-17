import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import ExecutionSample from '../views/ExecutionSample.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/execution'
  },
  {
    path: '/execution',
    name: 'execution',
    component: ExecutionSample
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
