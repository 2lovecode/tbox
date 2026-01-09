import { createMemoryHistory, createRouter } from 'vue-router'
import HomePage from '@/views/HomePage.vue'
import ImageCompression from '@/views/ImageCompression.vue'
import VideoConverter from '@/views/VideoConverter.vue'
import PasswordManage from '@/views/PasswordManage.vue'
import PDFToolbox from '@/views/PDFToolbox.vue'
import ScreenRuler from '@/views/ScreenRuler.vue'
import CodeFormatter from '@/views/CodeFormatter.vue'
import FileRecovery from '@/views/FileRecovery.vue'
import NetworkSpeedTest from '@/views/NetworkSpeedTest.vue'
import JsonTool from '@/views/JsonTool.vue'

const routes = [
  { path: '/', component: HomePage },
  { path: '/image-compression', component: ImageCompression },
  { path: '/video-converter', component: VideoConverter },
  { path: '/password-manage', component: PasswordManage },
  { path: '/pdf-toolbox', component: PDFToolbox },
  { path: '/screen-ruler', component: ScreenRuler },
  { path: '/code-formatter', component: CodeFormatter },
  { path: '/file-recovery', component: FileRecovery },
  { path: '/network-speed-test', component: NetworkSpeedTest },
  { path: '/json-tool', component: JsonTool }
]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})