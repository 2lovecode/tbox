import { createMemoryHistory, createRouter } from 'vue-router'
import HomePage from '@/views/HomePage.vue'

// 首屏 HomePage 保持静态导入(应用启动即渲染);
// 其余所有工具页改为动态 import,Vite 会按路由自动分片,
// 避免首屏一次性解析全部 37 个工具的代码。
const routes = [
  { path: '/', component: HomePage },
  { path: '/toolbox', component: () => import('@/views/ToolboxPage.vue') },
  { path: '/image-compression', component: () => import('@/views/ImageCompression.vue') },
  { path: '/video-converter', component: () => import('@/views/VideoConverter.vue') },
  { path: '/password-manage', component: () => import('@/views/PasswordManage.vue') },
  { path: '/pdf-toolbox', component: () => import('@/views/PDFToolbox.vue') },
  { path: '/screen-ruler', component: () => import('@/views/ScreenRuler.vue') },
  { path: '/code-formatter', component: () => import('@/views/CodeFormatter.vue') },
  { path: '/file-recovery', component: () => import('@/views/FileRecovery.vue') },
  { path: '/network-speed-test', component: () => import('@/views/NetworkSpeedTest.vue') },
  { path: '/json-tool', component: () => import('@/views/JsonTool.vue') },
  { path: '/base64-tool', component: () => import('@/views/Base64Tool.vue') },
  { path: '/hash-generator', component: () => import('@/views/HashGenerator.vue') },

  // 新增工具路由
  { path: '/json-to-entity', component: () => import('@/views/tools/JsonToEntity.vue') },
  { path: '/json-diff', component: () => import('@/views/tools/JsonDiff.vue') },
  { path: '/jwt-tool', component: () => import('@/views/tools/JwtTool.vue') },
  { path: '/regex-tester', component: () => import('@/views/tools/RegexTester.vue') },
  { path: '/timestamp-converter', component: () => import('@/views/tools/TimestampConverter.vue') },
  { path: '/http-request', component: () => import('@/views/tools/HttpRequest.vue') },
  { path: '/text-tools', component: () => import('@/views/tools/TextTools.vue') },
  { path: '/encoding-tools', component: () => import('@/views/tools/EncodingTools.vue') },
  { path: '/xml-tools', component: () => import('@/views/tools/XmlTools.vue') },
  { path: '/yaml-tools', component: () => import('@/views/tools/YamlTools.vue') },
  { path: '/gm-crypto', component: () => import('@/views/tools/GmCrypto.vue') },
  { path: '/sql-tools', component: () => import('@/views/tools/SqlTools.vue') },
  { path: '/database-tools', component: () => import('@/views/tools/DatabaseTools.vue') },
  { path: '/image-tools', component: () => import('@/views/tools/ImageTools.vue') },
  { path: '/csv-tools', component: () => import('@/views/tools/CsvTools.vue') },
  { path: '/log-analyzer', component: () => import('@/views/tools/LogAnalyzer.vue') },
  { path: '/color-tools', component: () => import('@/views/tools/ColorTools.vue') },
  { path: '/qrcode-tools', component: () => import('@/views/tools/QrcodeTools.vue') },
  { path: '/uuid-tools', component: () => import('@/views/tools/UuidTools.vue') },
  { path: '/cron-tools', component: () => import('@/views/tools/CronTools.vue') },
  { path: '/number-tools', component: () => import('@/views/tools/NumberTools.vue') },
  { path: '/charset-tools', component: () => import('@/views/tools/CharsetTools.vue') },
  { path: '/json-to-query', component: () => import('@/views/tools/JsonToQuery.vue') },
  { path: '/coordinate-tools', component: () => import('@/views/tools/CoordinateTools.vue') },
  { path: '/coordinate-visualizer', component: () => import('@/views/tools/CoordinateVisualizer.vue') }
]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})
