import {createRouter, createWebHashHistory, RouteRecordRaw} from 'vue-router'
import ExecutionSample from '../views/ExecutionSample.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/execution-sample'
  },
  {
    path: '/execution-sample',
    name: 'execution-sample',
    component: ExecutionSample
  }
]

const router = createRouter({
  history: createWebHashHistory(process.env.BASE_URL),
  routes
})

export default router
