import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import JFRView from '../views/JFRView.vue'
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
  },
  {
    path: '/jfr',
    name: 'jfr',
    component: JFRView
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
