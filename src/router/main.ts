import { createMemoryHistory, createRouter } from 'vue-router'
import HomePage from '@/views/HomePage.vue'
import ImageCompression from '@/views/ImageCompression.vue'

const routes = [
  { path: '/', component: HomePage },
  { path: '/image-compression', component: ImageCompression }
]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})