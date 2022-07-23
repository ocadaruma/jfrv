import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import JFRView from '@/views/JFRView.vue'
import HomeView from "@/views/HomeView.vue";

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    redirect: "/jfr",
  },
  {
    path: "/home",
    name: "home",
    component: HomeView
  },
  {
    path: "/jfr",
    name: "jfr",
    component: JFRView
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
