<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { formatShortcut, type ShortcutSpec } from '@/composables/useKeyboardShortcuts';
import { detectPlatform } from '@/utils/platform';

interface ShortcutEntry {
  /** Stable id so the overlay can scroll back to it if needed. */
  id: string;
  /** Group label rendered as a column heading. */
  group: '导航' | '工具' | '结果' | '面板';
  description: string;
  spec: ShortcutSpec;
  /** Same as the bind() option — default true. */
  crossPlatform?: boolean;
}

const isOpen = ref(false);

const platform = computed<'mac' | 'other'>(() =>
  detectPlatform() === 'mac' ? 'mac' : 'other',
);

const shortcuts: ShortcutEntry[] = [
  { id: 'spotlight', group: '导航', description: '打开 / 关闭 Spotlight 全局搜索', spec: { key: 'k', meta: true, ctrl: true } },
  { id: 'home', group: '导航', description: '返回首页', spec: { key: '[', meta: true, ctrl: true } },
  { id: 'theme', group: '面板', description: '切换浅色 / 深色主题', spec: { key: '\\', meta: true, ctrl: true } },
  { id: 'hints', group: '面板', description: '显示 / 隐藏快捷键面板', spec: { key: '?' } },
  { id: 'escape', group: '面板', description: '关闭弹窗', spec: { key: 'Escape' } },
  { id: 'enter', group: '工具', description: '提交 / 执行当前操作', spec: { key: 'Enter' } },
  { id: 'tab', group: '工具', description: '在结果 / 表单之间移动焦点', spec: { key: 'Tab' } },
  { id: 'copy', group: '结果', description: '复制当前结果', spec: { key: 'c', meta: true, ctrl: true } },
];

const grouped = computed(() => {
  const order: ShortcutEntry['group'][] = ['导航', '工具', '结果', '面板'];
  return order.map((group) => ({
    group,
    items: shortcuts.filter((s) => s.group === group),
  }));
});

function open() {
  isOpen.value = true;
}
function close() {
  isOpen.value = false;
}
function toggle() {
  isOpen.value = !isOpen.value;
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    event.preventDefault();
    close();
    return;
  }
  if (event.repeat) return;
  if (event.key === '?') {
    // Don't open the panel while the user is typing — the question mark is
    // legitimate input on chat / search fields.
    const target = event.target;
    if (target instanceof HTMLElement) {
      const tag = target.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || target.isContentEditable) return;
    }
    event.preventDefault();
    toggle();
  }
}

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('keydown', handleKeyDown);
  }
});
onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('keydown', handleKeyDown);
  }
});

defineExpose({ open, close, toggle });
</script>

<template>
  <Teleport to="body">
    <Transition name="shortcut-fade">
      <div v-if="isOpen" class="shortcut-overlay" role="dialog" aria-modal="true" aria-label="键盘快捷键" @click.self="close">
        <div class="shortcut-panel">
          <header class="shortcut-header">
            <h2>键盘快捷键</h2>
            <button class="close-btn" type="button" @click="close" aria-label="关闭">
              <i class="fas fa-times" aria-hidden="true"></i>
            </button>
          </header>

          <div class="shortcut-grid">
            <section v-for="group in grouped" :key="group.group" class="shortcut-group">
              <h3>{{ group.group }}</h3>
              <ul>
                <li v-for="entry in group.items" :key="entry.id">
                  <span class="description">{{ entry.description }}</span>
                  <kbd class="shortcut-key">{{ formatShortcut(entry.spec, { crossPlatform: entry.crossPlatform ?? true, platform: platform }) }}</kbd>
                </li>
              </ul>
            </section>
          </div>

          <footer class="shortcut-footer">
            <span>按 <kbd>?</kbd> 随时唤起本面板 · <kbd>Esc</kbd> 关闭</span>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.shortcut-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9500;
  padding: 24px;
  backdrop-filter: blur(4px);
}

.shortcut-panel {
  width: min(720px, 100%);
  max-height: min(80vh, 640px);
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.shortcut-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.shortcut-header h2 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}

.close-btn:hover {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}

.shortcut-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 20px 32px;
  padding: 20px 24px;
  overflow-y: auto;
}

.shortcut-group h3 {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: var(--text-secondary, #6c757d);
  margin: 0 0 10px 0;
}

.shortcut-group ul {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shortcut-group li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  font-size: 14px;
}

.description {
  flex: 1;
  color: var(--text-primary, #212529);
}

.shortcut-key {
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 6px;
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  white-space: nowrap;
}

.shortcut-footer {
  padding: 12px 24px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  background: var(--bg-secondary, #f8f9fa);
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
  display: flex;
  align-items: center;
  gap: 6px;
}

.shortcut-footer kbd {
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--bg-primary, #ffffff);
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.shortcut-fade-enter-active,
.shortcut-fade-leave-active {
  transition: opacity 0.18s ease;
}

.shortcut-fade-enter-from,
.shortcut-fade-leave-to {
  opacity: 0;
}

@media (prefers-color-scheme: dark) {
  .shortcut-overlay {
    background: rgba(0, 0, 0, 0.7);
  }
  .shortcut-key,
  .shortcut-footer kbd {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.1);
  }
}
</style>
