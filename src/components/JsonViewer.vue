<script lang="ts" setup>
import { ref, computed, watch, provide } from 'vue'
import JsonNode from './JsonNode.vue'

interface JsonNodeData {
  key: string
  value: any
  type: 'object' | 'array' | 'string' | 'number' | 'boolean' | 'null'
  level: number
  path: string
  children?: JsonNodeData[]
}

const props = defineProps<{
  json: string
}>()

const emit = defineEmits<{
  copy: [text: string]
}>()

const parsedData = ref<any>(null)
const error = ref<string | null>(null)
const expandedPaths = ref<Set<string>>(new Set(['root']))
const searchQuery = ref('')

// 切换展开/折叠函数 - 使用更安全的方式
const toggleExpand = (path: string) => {
  if (!expandedPaths.value) {
    expandedPaths.value = new Set(['root'])
  }
  if (expandedPaths.value.has(path)) {
    expandedPaths.value.delete(path)
  } else {
    expandedPaths.value.add(path)
  }
}

// 提供共享状态给子组件
provide('expandedPaths', expandedPaths)
provide('searchQuery', searchQuery)
provide('toggleExpand', toggleExpand)

// 解析JSON
watch(() => props.json, (newJson) => {
  if (!newJson || !newJson.trim()) {
    parsedData.value = null
    error.value = null
    expandedPaths.value = new Set(['root'])
    return
  }

  try {
    // 先尝试直接解析
    const parsed = JSON.parse(newJson)
    parsedData.value = parsed
    error.value = null
    // 重置展开状态
    expandedPaths.value = new Set(['root'])
    // 默认展开前2层
    expandToLevel(parsedData.value, 'root', 2)
  } catch (e: any) {
    error.value = e.message || 'JSON解析失败'
    parsedData.value = null
    expandedPaths.value = new Set(['root'])
  }
}, { immediate: true })

// 展开到指定层级
const expandToLevel = (data: any, path: string, maxLevel: number, currentLevel = 0) => {
  if (currentLevel >= maxLevel) return
  
  if (Array.isArray(data)) {
    data.forEach((item, index) => {
      const itemPath = `${path}[${index}]`
      expandedPaths.value.add(itemPath)
      if (typeof item === 'object' && item !== null) {
        expandToLevel(item, itemPath, maxLevel, currentLevel + 1)
      }
    })
  } else if (typeof data === 'object' && data !== null) {
    Object.keys(data).forEach(key => {
      const keyPath = path === 'root' ? key : `${path}.${key}`
      expandedPaths.value.add(keyPath)
      const value = data[key]
      if (typeof value === 'object' && value !== null) {
        expandToLevel(value, keyPath, maxLevel, currentLevel + 1)
      }
    })
  }
}

// 构建JSON树
const buildTree = (data: any, key = 'root', level = 0, path = 'root'): JsonNodeData | null => {
  try {
    if (data === null) {
      return {
        key,
        value: null,
        type: 'null',
        level,
        path
      }
    }

    // 确保字符串值不会被再次解析
    if (typeof data === 'string') {
      return {
        key,
        value: data,
        type: 'string',
        level,
        path
      }
    }

    const type = Array.isArray(data) ? 'array' : typeof data as any

    if (type === 'object') {
      const children: JsonNodeData[] = []
      Object.keys(data).forEach(k => {
        try {
          const childPath = path === 'root' ? k : `${path}.${k}`
          const child = buildTree(data[k], k, level + 1, childPath)
          if (child) {
            children.push(child)
          }
        } catch (e) {
          console.error('buildTree error for key:', k, e)
        }
      })
      return {
        key,
        value: data,
        type: 'object',
        level,
        path,
        children
      }
    }

    if (type === 'array') {
      const children: JsonNodeData[] = []
      data.forEach((item: any, index: number) => {
        try {
          const childPath = `${path}[${index}]`
          const child = buildTree(item, `[${index}]`, level + 1, childPath)
          if (child) {
            children.push(child)
          }
        } catch (e) {
          console.error('buildTree error for index:', index, e)
        }
      })
      return {
        key,
        value: data,
        type: 'array',
        level,
        path,
        children
      }
    }

    return {
      key,
      value: data,
      type,
      level,
      path
    }
  } catch (e) {
    console.error('buildTree error:', e, 'data:', data, 'key:', key)
    return null
  }
}

const jsonTree = computed(() => {
  try {
    if (!parsedData.value) return null
    console.log('Building jsonTree, parsedData:', parsedData.value)
    const result = buildTree(parsedData.value)
    console.log('jsonTree built:', result)
    return result
  } catch (e: any) {
    console.error('jsonTree computed error:', e)
    error.value = `构建JSON树失败: ${e.message}`
    return null
  }
})

// 展开所有
const expandAll = () => {
  const expandNode = (node: JsonNodeData) => {
    if (node.children) {
      expandedPaths.value.add(node.path)
      node.children.forEach(expandNode)
    }
  }
  if (jsonTree.value) {
    expandNode(jsonTree.value)
  }
}

// 折叠所有
const collapseAll = () => {
  expandedPaths.value.clear()
  expandedPaths.value.add('root')
}

// 复制JSON
const copyJson = () => {
  emit('copy', props.json)
}
</script>

<template>
  <div class="json-viewer">
    <div class="viewer-toolbar" v-if="parsedData && jsonTree">
      <div class="toolbar-left">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索字段或值..."
          class="search-input"
        />
      </div>
      <div class="toolbar-right">
        <button @click="expandAll" class="toolbar-btn" title="展开所有">
          <i class="fas fa-expand-arrows-alt"></i>
        </button>
        <button @click="collapseAll" class="toolbar-btn" title="折叠所有">
          <i class="fas fa-compress-arrows-alt"></i>
        </button>
        <button @click="copyJson" class="toolbar-btn" title="复制">
          <i class="fas fa-copy"></i>
        </button>
      </div>
    </div>

    <div v-if="error" class="json-error">
      <i class="fas fa-exclamation-triangle"></i>
      {{ error }}
    </div>

    <div v-else-if="!parsedData" class="json-empty">
      暂无JSON数据
    </div>

    <div v-else-if="!jsonTree" class="json-empty">
      正在构建JSON树...
    </div>

    <div v-else class="json-tree">
      <JsonNode :node="jsonTree" />
    </div>
  </div>
</template>

<style scoped>
.json-viewer {
  background: #1e1e1e;
  border-radius: 8px;
  padding: 15px;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  max-height: 600px;
  overflow: auto;
  color: #d4d4d4;
}

.viewer-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #3a3a3a;
}

.toolbar-left {
  flex: 1;
  margin-right: 15px;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  background: #2d2d2d;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #d4d4d4;
  font-size: 14px;
}

.search-input:focus {
  outline: none;
  border-color: #0ea5e9;
}

.toolbar-right {
  display: flex;
  gap: 8px;
}

.toolbar-btn {
  padding: 6px 12px;
  background: #2d2d2d;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  color: #d4d4d4;
  cursor: pointer;
  transition: all 0.2s;
}

.toolbar-btn:hover {
  background: #3a3a3a;
  border-color: #0ea5e9;
  color: #0ea5e9;
}

.json-error {
  color: #f87171;
  padding: 20px;
  text-align: center;
}

.json-empty {
  color: #6b7280;
  padding: 20px;
  text-align: center;
}

.json-tree {
  user-select: text;
}

/* 滚动条样式 */
.json-viewer::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.json-viewer::-webkit-scrollbar-track {
  background: #1e1e1e;
}

.json-viewer::-webkit-scrollbar-thumb {
  background: #3a3a3a;
  border-radius: 4px;
}

.json-viewer::-webkit-scrollbar-thumb:hover {
  background: #4a4a4a;
}
</style>
