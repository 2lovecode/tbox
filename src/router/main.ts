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
import Base64Tool from '@/views/Base64Tool.vue'
import HashGenerator from '@/views/HashGenerator.vue'

// 新增工具
import JsonToEntity from '@/views/tools/JsonToEntity.vue'
import JsonDiff from '@/views/tools/JsonDiff.vue'
import JwtTool from '@/views/tools/JwtTool.vue'
import RegexTester from '@/views/tools/RegexTester.vue'
import TimestampConverter from '@/views/tools/TimestampConverter.vue'
import HttpRequest from '@/views/tools/HttpRequest.vue'
import TextTools from '@/views/tools/TextTools.vue'
import EncodingTools from '@/views/tools/EncodingTools.vue'
import XmlTools from '@/views/tools/XmlTools.vue'
import YamlTools from '@/views/tools/YamlTools.vue'
import GmCrypto from '@/views/tools/GmCrypto.vue'
import SqlTools from '@/views/tools/SqlTools.vue'
import DatabaseTools from '@/views/tools/DatabaseTools.vue'
import ImageTools from '@/views/tools/ImageTools.vue'
import CsvTools from '@/views/tools/CsvTools.vue'
import LogAnalyzer from '@/views/tools/LogAnalyzer.vue'
import ColorTools from '@/views/tools/ColorTools.vue'
import QrcodeTools from '@/views/tools/QrcodeTools.vue'
import UuidTools from '@/views/tools/UuidTools.vue'
import CronTools from '@/views/tools/CronTools.vue'
import NumberTools from '@/views/tools/NumberTools.vue'
import CharsetTools from '@/views/tools/CharsetTools.vue'

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
  { path: '/json-tool', component: JsonTool },
  { path: '/base64-tool', component: Base64Tool },
  { path: '/hash-generator', component: HashGenerator },

  // 新增工具路由
  { path: '/json-to-entity', component: JsonToEntity },
  { path: '/json-diff', component: JsonDiff },
  { path: '/jwt-tool', component: JwtTool },
  { path: '/regex-tester', component: RegexTester },
  { path: '/timestamp-converter', component: TimestampConverter },
  { path: '/http-request', component: HttpRequest },
  { path: '/text-tools', component: TextTools },
  { path: '/encoding-tools', component: EncodingTools },
  { path: '/xml-tools', component: XmlTools },
  { path: '/yaml-tools', component: YamlTools },
  { path: '/gm-crypto', component: GmCrypto },
  { path: '/sql-tools', component: SqlTools },
  { path: '/database-tools', component: DatabaseTools },
  { path: '/image-tools', component: ImageTools },
  { path: '/csv-tools', component: CsvTools },
  { path: '/log-analyzer', component: LogAnalyzer },
  { path: '/color-tools', component: ColorTools },
  { path: '/qrcode-tools', component: QrcodeTools },
  { path: '/uuid-tools', component: UuidTools },
  { path: '/cron-tools', component: CronTools },
  { path: '/number-tools', component: NumberTools },
  { path: '/charset-tools', component: CharsetTools }
]

export const router = createRouter({
  history: createMemoryHistory(),
  routes,
})