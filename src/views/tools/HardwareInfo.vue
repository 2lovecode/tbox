<script lang="ts" setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import PageHeader from '@/components/PageHeader.vue'
import { toast } from '@/utils/toast'

interface DiskRow {
  name: string
  kind: string
  mountPoint: string
  fileSystem: string
  totalBytes: number
  availableBytes: number
  isRemovable: boolean
  isReadOnly: boolean
}

interface NetRow {
  interface: string
  receivedBytes: number
  transmittedBytes: number
  packetsReceived: number
  packetsTransmitted: number
  mtu: number
  macAddress: string
}

interface HardwarePayload {
  hostname: string
  os: {
    type: string
    version: string
    edition?: string | null
    codename?: string | null
    architecture?: string | null
    bitness: string
  }
  kernel: string
  longOsVersion: string
  osName: string
  osVersionShort: string
  distributionId: string
  cpuArch: string
  bootTimeUnix: number
  uptimeSeconds: number
  cpu: {
    brand: string
    logicalCores: number
    physicalCores: number
    frequencyMhz: number
  }
  memory: {
    totalBytes: number
    availableBytes: number
    usedBytes: number
  }
  swap: {
    totalBytes: number
    usedBytes: number
  }
  disks: DiskRow[]
  networks: NetRow[]
  loadAverage?: { one: number; five: number; fifteen: number } | null
}

const loading = ref(false)
const data = ref<HardwarePayload | null>(null)
const errorMsg = ref('')

function formatBytes(n: number): string {
  if (n === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let v = n
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024
    i++
  }
  return `${v.toFixed(i === 0 ? 0 : 2)} ${units[i]}`
}

function formatUptime(sec: number): string {
  if (sec < 60) return `${sec} 秒`
  const d = Math.floor(sec / 86400)
  const h = Math.floor((sec % 86400) / 3600)
  const m = Math.floor((sec % 3600) / 60)
  const parts: string[] = []
  if (d) parts.push(`${d} 天`)
  if (h) parts.push(`${h} 小时`)
  if (m || parts.length === 0) parts.push(`${m} 分钟`)
  return parts.join(' ')
}

const bootTimeText = computed(() => {
  if (!data.value?.bootTimeUnix) return '—'
  const d = new Date(data.value.bootTimeUnix * 1000)
  return isNaN(d.getTime()) ? '—' : d.toLocaleString()
})

const memPercent = computed(() => {
  if (!data.value) return 0
  const t = data.value.memory.totalBytes
  if (!t) return 0
  return Math.round((data.value.memory.usedBytes / t) * 1000) / 10
})

async function refresh() {
  loading.value = true
  errorMsg.value = ''
  try {
    const res = await invoke<HardwarePayload>('get_hardware_info')
    data.value = res
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    errorMsg.value = msg
    toast.error(msg)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="hardware-info">
    <PageHeader title="硬件信息" description="查看本机 CPU、内存、磁盘、网络与操作系统信息（Windows / macOS / Linux）。" />

    <div class="toolbar">
      <button type="button" class="btn-primary" :disabled="loading" @click="refresh">
        {{ loading ? '读取中…' : '刷新' }}
      </button>
    </div>

    <p v-if="errorMsg" class="error">{{ errorMsg }}</p>

    <div v-if="data" class="panels">
      <section class="panel">
        <h3>主机与系统</h3>
        <dl class="kv">
          <dt>主机名</dt>
          <dd>{{ data.hostname }}</dd>
          <dt>系统类型</dt>
          <dd>{{ data.os.type }}</dd>
          <dt>版本</dt>
          <dd>{{ data.os.version }}</dd>
          <dt>详细版本</dt>
          <dd>{{ data.longOsVersion }}</dd>
          <dt>内核</dt>
          <dd>{{ data.kernel }}</dd>
          <dt>架构</dt>
          <dd>{{ data.cpuArch }}{{ data.os.architecture ? `（${data.os.architecture}）` : '' }}</dd>
          <dt>位数</dt>
          <dd>{{ data.os.bitness }}</dd>
          <dt v-if="data.os.edition">发行版 / 版本名称</dt>
          <dd v-if="data.os.edition">{{ data.os.edition }}</dd>
          <dt v-if="data.os.codename">代号</dt>
          <dd v-if="data.os.codename">{{ data.os.codename }}</dd>
          <dt>发行版 ID</dt>
          <dd>{{ data.distributionId }}</dd>
          <dt>启动时间</dt>
          <dd>{{ bootTimeText }}</dd>
          <dt>运行时长</dt>
          <dd>{{ formatUptime(data.uptimeSeconds) }}</dd>
        </dl>
      </section>

      <section class="panel">
        <h3>处理器</h3>
        <dl class="kv">
          <dt>型号</dt>
          <dd class="wide">{{ data.cpu.brand || '—' }}</dd>
          <dt>物理核心</dt>
          <dd>{{ data.cpu.physicalCores }}</dd>
          <dt>逻辑处理器</dt>
          <dd>{{ data.cpu.logicalCores }}</dd>
          <dt>基准频率</dt>
          <dd>{{ data.cpu.frequencyMhz > 0 ? `${data.cpu.frequencyMhz} MHz` : '—' }}</dd>
        </dl>
      </section>

      <section class="panel">
        <h3>内存与交换</h3>
        <dl class="kv">
          <dt>物理内存</dt>
          <dd>
            {{ formatBytes(data.memory.usedBytes) }} / {{ formatBytes(data.memory.totalBytes) }}
            <span class="muted">（约 {{ memPercent }}% 已用）</span>
          </dd>
          <dt>可用内存</dt>
          <dd>{{ formatBytes(data.memory.availableBytes) }}</dd>
          <dt>交换分区</dt>
          <dd>
            <template v-if="data.swap.totalBytes > 0">
              {{ formatBytes(data.swap.usedBytes) }} / {{ formatBytes(data.swap.totalBytes) }}
            </template>
            <template v-else>未配置或未报告</template>
          </dd>
        </dl>
      </section>

      <section v-if="data.loadAverage" class="panel">
        <h3>负载（Unix）</h3>
        <dl class="kv">
          <dt>1 / 5 / 15 分钟</dt>
          <dd>
            {{ data.loadAverage.one.toFixed(2) }} /
            {{ data.loadAverage.five.toFixed(2) }} /
            {{ data.loadAverage.fifteen.toFixed(2) }}
          </dd>
        </dl>
      </section>

      <section class="panel panel-wide">
        <h3>磁盘卷</h3>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>挂载点</th>
                <th>名称</th>
                <th>类型</th>
                <th>文件系统</th>
                <th>容量</th>
                <th>可用</th>
                <th>可移动</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(d, i) in data.disks" :key="i">
                <td>{{ d.mountPoint }}</td>
                <td>{{ d.name }}</td>
                <td><code>{{ d.kind }}</code></td>
                <td>{{ d.fileSystem }}</td>
                <td>{{ formatBytes(d.totalBytes) }}</td>
                <td>{{ formatBytes(d.availableBytes) }}</td>
                <td>{{ d.isRemovable ? '是' : '否' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="panel panel-wide">
        <h3>网络接口</h3>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                <th>接口</th>
                <th>MAC</th>
                <th>MTU</th>
                <th>累计接收</th>
                <th>累计发送</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(n, i) in data.networks" :key="i">
                <td>{{ n.interface }}</td>
                <td><code class="mac">{{ n.macAddress }}</code></td>
                <td>{{ n.mtu }}</td>
                <td>{{ formatBytes(n.receivedBytes) }}</td>
                <td>{{ formatBytes(n.transmittedBytes) }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <p v-else-if="!loading && !errorMsg" class="hint">点击「刷新」读取本机硬件信息。</p>
  </div>
</template>

<style scoped>
.hardware-info {
  padding: 1rem 1.25rem 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.toolbar {
  margin: 1rem 0;
}

.btn-primary {
  padding: 0.5rem 1.25rem;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  background: linear-gradient(135deg, #4361ee, #4895ef);
  color: #fff;
  font-size: 0.95rem;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error {
  color: #e63946;
  margin: 0.5rem 0;
}

.hint {
  color: #666;
  margin-top: 1rem;
}

.panels {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1rem;
}

.panel {
  background: var(--card-bg, rgba(255, 255, 255, 0.06));
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.12));
  border-radius: 12px;
  padding: 1rem 1.25rem;
}

.panel-wide {
  grid-column: 1 / -1;
}

.panel h3 {
  margin: 0 0 0.75rem;
  font-size: 1.05rem;
  font-weight: 600;
}

.kv {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 0.35rem 1rem;
  margin: 0;
  font-size: 0.9rem;
}

.kv dt {
  margin: 0;
  color: var(--muted, #888);
  white-space: nowrap;
}

.kv dd {
  margin: 0;
  word-break: break-word;
}

.kv dd.wide {
  grid-column: 2;
}

.muted {
  color: var(--muted, #888);
  font-size: 0.85rem;
}

.table-wrap {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
}

th,
td {
  text-align: left;
  padding: 0.5rem 0.6rem;
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
}

th {
  color: var(--muted, #888);
  font-weight: 500;
}

code {
  font-size: 0.8rem;
}

code.mac {
  font-size: 0.75rem;
}
</style>
