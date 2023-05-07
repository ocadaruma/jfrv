import {createRouter, createWebHashHistory, RouteRecordRaw} from 'vue-router'
import ExecutionSample from '@/views/ExecutionSample.vue'
import JvmBlockingMonitor from '@/views/JvmBlockingMonitor.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/execution-sample'
  },
  {
    path: '/execution-sample',
    name: 'execution-sample',
    component: ExecutionSample
  },
  {
    path: '/jbm',
    name: 'jbm',
    component: JvmBlockingMonitor
  },
]

const router = createRouter({
  history: createWebHashHistory(process.env.BASE_URL),
  routes
})

export default router
