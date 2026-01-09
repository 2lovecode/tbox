<script lang="ts" setup>
import { inject, computed, ref } from 'vue'
import type { Ref } from 'vue'

interface JsonNode {
  key: string
  value: any
  type: 'object' | 'array' | 'string' | 'number' | 'boolean' | 'null'
  level: number
  path: string
  children?: JsonNode[]
}

const props = defineProps<{
  node: JsonNode
}>()

// 更安全的inject，使用computed来确保值存在
const expandedPaths = inject<Ref<Set<string>> | undefined>('expandedPaths', undefined)
const searchQuery = inject<Ref<string> | undefined>('searchQuery', undefined)
const toggleExpand = inject<((path: string) => void) | undefined>('toggleExpand', undefined)

const isExpanded = computed(() => {
  try {
    if (!expandedPaths?.value) return false
    return expandedPaths.value.has(props.node.path)
  } catch {
    return false
  }
})

const hasChildren = computed(() => props.node.children && props.node.children.length > 0)

const matchesSearch = computed(() => {
  try {
    if (!searchQuery?.value) return true
    const query = searchQuery.value.toLowerCase()
    const keyMatch = props.node.key.toLowerCase().includes(query)
    if (props.node.type === 'string' || props.node.type === 'number' || props.node.type === 'boolean') {
      const valueMatch = String(props.node.value).toLowerCase().includes(query)
      return keyMatch || valueMatch
    }
    return keyMatch
  } catch {
    return true
  }
})

const highlightText = (text: string): string => {
  try {
    if (!searchQuery?.value) return text
    const query = searchQuery.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    const regex = new RegExp(`(${query})`, 'gi')
    return text.replace(regex, '<mark>$1</mark>')
  } catch {
    return text
  }
}

const formatValue = (): string => {
  const node = props.node
  if (node.type === 'string') {
    return `"${node.value}"`
  }
  if (node.type === 'null') {
    return 'null'
  }
  if (node.type === 'boolean') {
    return String(node.value)
  }
  return String(node.value)
}

const getTypeColor = (type: string): string => {
  const colors: Record<string, string> = {
    string: '#0ea5e9',
    number: '#10b981',
    boolean: '#f59e0b',
    null: '#6b7280',
    object: '#8b5cf6',
    array: '#ec4899'
  }
  return colors[type] || '#6b7280'
}

const handleToggle = () => {
  try {
    if (hasChildren.value && toggleExpand) {
      toggleExpand(props.node.path)
    }
  } catch (e) {
    console.error('Toggle expand error:', e)
  }
}
</script>

<template>
  <div class="json-node" :class="{ 'has-match': matchesSearch }" :style="{ paddingLeft: (node.level * 20) + 'px' }">
    <div 
      class="node-line" 
      @click="handleToggle" 
      :style="{ cursor: hasChildren ? 'pointer' : 'default' }"
    >
      <span v-if="hasChildren" class="expand-icon">
        <i :class="isExpanded ? 'fas fa-chevron-down' : 'fas fa-chevron-right'"></i>
      </span>
      <span v-else class="expand-placeholder"></span>
      
      <span class="node-key" v-if="node.key !== 'root'">
        <span class="key-text" v-html="highlightText(node.key)"></span>
        <span class="key-separator">:</span>
      </span>
      
      <span v-if="!hasChildren" class="node-value">
        <span class="type-badge" :style="{ color: getTypeColor(node.type) }">{{ node.type }}</span>
        <span class="value-text" v-html="highlightText(formatValue())"></span>
      </span>
      
      <span v-else class="node-type">
        <span class="type-badge" :style="{ color: getTypeColor(node.type) }">
          {{ node.type === 'object' ? '{' : '[' }}{{ node.children?.length ?? 0 }}{{ node.type === 'object' ? '}' : ']' }}
        </span>
      </span>
    </div>
    
    <div v-if="hasChildren && isExpanded" class="node-children">
      <JsonNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
      />
    </div>
  </div>
</template>

<style scoped>
.json-node {
  position: relative;
}

.json-node.has-match {
  background: rgba(14, 165, 233, 0.1);
  border-left: 2px solid #0ea5e9;
  padding-left: 8px;
  margin-left: -8px;
}

.node-line {
  display: flex;
  align-items: center;
  padding: 4px 0;
  transition: background 0.2s;
}

.node-line:hover {
  background: rgba(255, 255, 255, 0.05);
}

.expand-icon {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-right: 6px;
  color: #6b7280;
  transition: color 0.2s;
}

.expand-icon:hover {
  color: #0ea5e9;
}

.expand-placeholder {
  width: 16px;
  margin-right: 6px;
}

.node-key {
  margin-right: 8px;
}

.key-text {
  color: #9cdcfe;
  font-weight: 500;
}

.key-separator {
  color: #6b7280;
  margin: 0 4px;
}

.node-value {
  display: flex;
  align-items: center;
  gap: 8px;
}

.value-text {
  color: #ce9178;
}

.node-type {
  display: flex;
  align-items: center;
}

.type-badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.1);
  font-weight: 600;
  text-transform: uppercase;
}

.node-children {
  margin-left: 0;
}

:deep(mark) {
  background: #fbbf24;
  color: #1e1e1e;
  padding: 2px 4px;
  border-radius: 3px;
  font-weight: 600;
}
</style>
