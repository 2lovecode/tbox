<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { onBeforeUnmount } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Tool, Category } from "@/types/tools";
import { useToolStore  }  from  "@/stores/tools";
import SideBar from "@/layout/SideBar.vue";
import SpotlightSearch from "@/components/SpotlightSearch.vue";
import { useSearchStore } from "@/stores/search";
import { RouterView, useRoute, useRouter } from "vue-router";
import Toast from "@/components/Toast.vue";
import ShortcutHints from "@/components/ShortcutHints.vue";
import ConfirmModal from "@/components/ConfirmModal.vue";
import { useTheme } from "@/composables/useTheme";
import { useOnlineStatus } from "@/composables/useOnlineStatus";
import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";
import { formatShortcut } from "@/composables/useKeyboardShortcuts";
import { detectPlatform } from "@/utils/platform";
import { useSidebarWidth } from "@/composables/useSidebarWidth";

const store  = useToolStore()
const searchStore = useSearchStore()
const route = useRoute()
const router = useRouter()

function openSettingsPage() {
  if (route.path.startsWith('/settings')) {
    if (window.history.length > 1) router.back()
    else void router.push('/')
    return
  }
  void router.push('/settings')
}
const { isDark, toggleTheme } = useTheme()
const { isOnline } = useOnlineStatus()
const shortcuts = useKeyboardShortcuts()

// 不参与 keep-alive 缓存的工具:这些工具靠 onUnmounted 清理定时器/
// 全局事件监听,keep-alive 下切走不会触发卸载,会导致后台定时器空转、
// document 监听泄漏等问题。其余无副作用的工具进入缓存,切换时保留输入。
const excludedFromCache = ['NetworkSpeedTest', 'ScreenRuler', 'FileRecovery', 'VideoConverter']

// 工具数据
const tools = ref<Tool[]>([]);
const isLoading = ref(true);

// Subscribe to the global Spotlight shortcut event emitted by the Rust
// side. The unlisten handle is captured for cleanup on unmount so HMR
// can't leave stale listeners behind.
let unlistenSpotlight: (() => void) | null = null;

// Global keyboard shortcuts. `useKeyboardShortcuts` auto-cleans on scope
// dispose, so we just declare them once. Cmd/Ctrl+K toggles Spotlight,
// Cmd/Ctrl+[ jumps to the homepage (mirrors the macOS "back" convention),
// Cmd/Ctrl+\ toggles theme, and `?` (handled inside ShortcutHints itself)
// surfaces the panel.
shortcuts.bind('k', { meta: true, ctrl: true }, () => {
  searchStore.toggle();
});
shortcuts.bind('[', { meta: true, ctrl: true }, () => {
  if (route.path !== '/') router.push('/');
});
shortcuts.bind('\\', { meta: true, ctrl: true }, () => {
  toggleTheme();
});
// Cmd/Ctrl+, opens the Settings page (mirrors the macOS convention).
shortcuts.bind(',', { meta: true, ctrl: true }, () => {
  openSettingsPage();
});

// 加载categories
// 加载tools
onMounted(async () => {
  unlistenSpotlight = await listen('spotlight:toggle', () => {
    searchStore.toggle();
  });

  isLoading.value = true;
  try {
    const [categoriesRes, toolsRes] = await Promise.all([
      invoke('get_categories').catch((error) => {
        console.error('Failed to load categories:', error);
        return [];
      }),
      invoke('get_all_tools').catch((error) => {
        console.error('Failed to load tools:', error);
        return [];
      })
    ]);

    if (categoriesRes && Array.isArray(categoriesRes)) {
      const fetchCategories = (categoriesRes as Array<Category>).map((item: Category) => ({
        id: item.id,
        name: item.name,
        icon: item.icon,
        count: item.count,
      }))
      store.setCategories(fetchCategories)
    }

    if (toolsRes && Array.isArray(toolsRes)) {
      const fetchTools = (toolsRes as Array<Tool>).map((item: Tool) => ({
        id: item.id,
        name: item.name,
        description: item.description,
        icon: item.icon,
        category: item.category,
        tags: item.tags,
        gradient: item.gradient,
      }))
      store.setTools(fetchTools)
      tools.value = fetchTools;
    }
  } catch (error) {
    console.error('Error loading data:', error);
  } finally {
    isLoading.value = false;
  }
})

// 仅对话相关页显示会话侧栏；工具箱 / 设置 / 具体工具页不显示
const showSidebar = computed(
  () =>
    route.path === '/' ||
    route.path.startsWith('/agent-runs/'),
)

const {
  cssWidth: sidebarCssWidth,
  dragging: sidebarDragging,
  startResize: startSidebarResize,
  ariaValue: sidebarAria,
} = useSidebarWidth(showSidebar)

const containerStyle = computed(() =>
  showSidebar.value
    ? ({ '--sidebar-width': sidebarCssWidth.value } as Record<string, string>)
    : undefined,
)

// Detect platform once for the keyboard-shortcut hint in the header.
const isMac = detectPlatform() === 'mac';
const spotlightHint = formatShortcut(
  { key: 'k', meta: true, ctrl: true },
  { platform: isMac ? 'mac' : 'other' },
);

onBeforeUnmount(() => {
  if (unlistenSpotlight) {
    unlistenSpotlight();
    unlistenSpotlight = null;
  }
});
</script>

<template>
      <div
        class="container"
        :class="{
          'no-sidebar': !showSidebar,
          'dark-mode': isDark,
          'sidebar-dragging': sidebarDragging,
        }"
        :style="containerStyle"
      >
        <header>
          <div class="logo" @click="$router.push('/')" style="cursor: pointer;">
            <div class="logo-icon">
              <i class="fas fa-toolbox"></i>
            </div>
            <div class="logo-text">T<span>Box</span></div>
          </div>
          <div class="header-actions">
            <span
              v-if="!isOnline"
              class="offline-badge"
              role="status"
              aria-live="polite"
              title="当前为离线状态，网络工具可能不可用"
            >
              <span class="offline-dot" aria-hidden="true"></span>
              <span class="offline-label">离线</span>
            </span>
            <button
              type="button"
              class="spotlight-trigger"
              :title="`打开搜索面板 (${spotlightHint})`"
              @click="searchStore.toggle()"
            >
              <i class="fas fa-search"></i>
              <span class="spotlight-trigger-label">搜索工具…</span>
              <kbd class="spotlight-trigger-kbd">{{ spotlightHint }}</kbd>
            </button>
            <button
              type="button"
              class="theme-toggle toolbox-toggle"
              :class="{ active: route.path === '/toolbox' }"
              title="工具箱"
              aria-label="打开工具箱"
              @click="router.push('/toolbox')"
            >
              <i class="fas fa-th-large"></i>
            </button>
            <button @click="toggleTheme" class="theme-toggle" :title="isDark ? '切换到浅色模式' : '切换到深色模式'">
              <i :class="isDark ? 'fas fa-sun' : 'fas fa-moon'"></i>
            </button>
            <button
              type="button"
              class="theme-toggle settings-toggle"
              title="设置"
              aria-label="打开设置"
              @click="openSettingsPage()"
            >
              <i class="fas fa-sliders"></i>
            </button>
          </div>
        </header>
      <SideBar v-if="showSidebar" />
      <div
        v-if="showSidebar"
        class="sidebar-resizer"
        role="separator"
        aria-orientation="vertical"
        aria-label="调整侧栏宽度"
        :aria-valuenow="sidebarAria.now"
        :aria-valuemin="sidebarAria.min"
        :aria-valuemax="sidebarAria.max"
        @pointerdown="startSidebarResize"
      ></div>
      <div class="main-wrapper">
        <Transition name="fade" mode="out-in">
          <RouterView v-if="!isLoading" v-slot="{ Component }">
            <KeepAlive :max="12" :exclude="excludedFromCache">
              <component :is="Component" />
            </KeepAlive>
          </RouterView>
          <div v-else class="loading-container">
            <div class="loading-spinner"></div>
            <p>加载中...</p>
          </div>
        </Transition>
      </div>
        <Toast />
        <SpotlightSearch />
        <ShortcutHints />
        <ConfirmModal />
      </div>
</template>

<style>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--text-secondary) 35%, transparent) transparent;
  }
  
  :root {
    --primary: #4361ee;
    --secondary: #3f37c9;
    --accent: #4895ef;
    --light: #f8f9fa;
    --dark: #212529;
    --gray: #6c757d;
    --success: #4cc9f0;
    --border-radius: 12px;
    --shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
    --transition: all 0.3s ease;
    --bg-primary: #ffffff;
    --bg-secondary: #f5f7fa;
    --bg-tertiary: #e4edf5;
    --surface-1: #ffffff;
    --surface-2: #f8fafc;
    --surface-elevated: #ffffff;
    --text-primary: #212529;
    --text-secondary: #6c757d;
    --border-color: rgba(0, 0, 0, 0.1);
    /* 壳层统一：柔和分隔线 + 水平 gutter */
    --shell-divider: color-mix(in srgb, var(--border-color) 72%, transparent);
    --shell-gutter: 16px;
    --control-radius: 8px;
    /* 侧栏 / 聊天列：相对视口自适应；侧栏实际宽度由拖拽写入 --sidebar-width */
    --sidebar-width-min: max(180px, 12vw);
    --sidebar-width-max: min(300px, 24vw);
    --sidebar-width: clamp(var(--sidebar-width-min), 15vw, var(--sidebar-width-max));
    --chat-width-min: max(280px, 28vw);
    --chat-width-max: min(840px, 62vw);
    --header-control-h: 32px;
    /* 聊天底栏与侧栏工具箱共用，保证顶部分隔线对齐 */
    --chat-dock-pad-top: 10px;
    --chat-dock-pad-bottom: 12px;
    --chat-dock-control-h: 40px;
    /* 代码块：随主题抬升一层，避免全黑卡片 */
    --code-bg: color-mix(in srgb, var(--bg-tertiary) 42%, var(--bg-primary));
    --code-fg: var(--text-primary);
    --code-muted: var(--text-secondary);
    --code-border: var(--shell-divider);
    --warning: #b45309;
    --danger: #dc2626;
    --success: #16a34a;
  }

  /* ---- Markdown 代码块（随主题，聊天与轨迹页共用） ---- */
  .md-content .md-code-block {
    position: relative;
    margin: 8px 0;
    max-width: 100%;
    box-sizing: border-box;
    padding: 28px 12px 12px;
    border-radius: var(--control-radius, 8px);
    background: var(--code-bg);
    overflow-x: auto;
    border: 1px solid var(--code-border);
  }

  .md-content .md-code-block code {
    background: transparent;
    padding: 0;
    color: var(--code-fg);
    font-size: 12.5px;
    line-height: 1.55;
    display: block;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .md-content .md-code-lang {
    position: absolute;
    top: 8px;
    left: 12px;
    right: auto;
    font-size: 10.5px;
    letter-spacing: 0.5px;
    text-transform: uppercase;
    color: var(--code-muted);
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    pointer-events: none;
  }

  .md-content .md-code-block .hljs {
    background: transparent !important;
    padding: 0;
    color: var(--code-fg);
  }

  /* 代码块复制按钮：固定右上角 */
  .md-content .md-code-copy {
    position: absolute;
    top: 6px;
    right: 8px;
    left: auto;
    z-index: 2;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: 6px;
    background: color-mix(in srgb, var(--text-secondary) 12%, transparent);
    color: var(--code-muted);
    font-size: 11px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    opacity: 1;
    transition: background 0.15s ease, color 0.15s ease;
    padding: 0;
  }

  .md-content .md-code-copy:hover,
  .md-content .md-code-copy:focus-visible {
    background: color-mix(in srgb, var(--text-secondary) 22%, transparent);
    color: var(--code-fg);
  }

  .md-content .md-code-copy.copied {
    color: var(--success);
  }

  /* html.dark 与 .dark-mode 同步，保证 body / 弹层 / 各页共用同一套 token */
  html.dark,
  html.dark-mode,
  .dark-mode {
    --bg-primary: #0f1115;
    --bg-secondary: #151820;
    --bg-tertiary: #1c2029;
    --surface-1: #151820;
    --surface-2: #1c2029;
    --surface-elevated: #222733;
    --text-primary: #e8eaed;
    --text-secondary: #9aa0a6;
    --border-color: rgba(255, 255, 255, 0.08);
    --shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
    --shell-divider: color-mix(in srgb, var(--border-color) 85%, transparent);
    --code-bg: #1a1d26;
    --code-fg: #d7dbe3;
    --code-muted: #8b919a;
    --code-border: var(--shell-divider);
    --warning: #fbbf24;
    --danger: #f87171;
    --success: #4ade80;
    color-scheme: dark;
  }

  html,
  body,
  #app {
    height: 100%;
    margin: 0;
  }

  body {
    background: var(--bg-primary);
    color: var(--text-primary);
    height: 100%;
    overflow: hidden;
    padding: 0;
    transition: background 0.3s ease, color 0.3s ease;
  }

  /* 全局细滚动条：WebKit */
  *::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  *::-webkit-scrollbar-track {
    background: transparent;
  }

  *::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--text-secondary) 28%, transparent);
    border-radius: 999px;
    border: 2px solid transparent;
    background-clip: padding-box;
  }

  *::-webkit-scrollbar-thumb:hover {
    background: color-mix(in srgb, var(--text-secondary) 48%, transparent);
    border: 2px solid transparent;
    background-clip: padding-box;
  }

  *::-webkit-scrollbar-corner {
    background: transparent;
  }
  
  .container {
    max-width: none;
    width: 100%;
    margin: 0;
    height: 100%;
    max-height: 100%;
    min-height: 0;
    display: grid;
    grid-template-columns: var(--sidebar-width) 5px minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    gap: 0;
    overflow: hidden;
  }

  .container.no-sidebar {
    grid-template-columns: 1fr;
  }

  .container.no-sidebar .main-wrapper {
    grid-column: 1;
    grid-row: 2;
    max-width: 100%;
  }

  .container > aside {
    grid-column: 1;
    grid-row: 2;
    min-height: 0;
    overflow: hidden;
    position: relative;
  }

  .main-wrapper {
    grid-column: 3;
    grid-row: 2;
    min-height: 0;
    height: 100%;
    overflow: hidden;
    background: transparent;
    display: flex;
    flex-direction: column;
    align-items: stretch;
  }

  /* 侧栏拖拽分隔条 */
  .sidebar-resizer {
    grid-row: 2;
    grid-column: 2;
    position: relative;
    z-index: 6;
    width: 100%;
    height: 100%;
    cursor: col-resize;
    touch-action: none;
    user-select: none;
    background: transparent;
  }

  .sidebar-resizer::after {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 1px;
    transform: translateX(-50%);
    background: var(--shell-divider);
    pointer-events: none;
    transition: background 0.12s ease, width 0.12s ease;
  }

  .sidebar-resizer:hover::after,
  .sidebar-resizer:focus-visible::after,
  .container.sidebar-dragging .sidebar-resizer::after {
    width: 2px;
    background: color-mix(in srgb, var(--primary) 55%, var(--shell-divider));
  }

  .container.sidebar-dragging {
    cursor: col-resize;
  }

  .container.sidebar-dragging .main-wrapper {
    pointer-events: none;
  }

  .main-wrapper > * {
    min-height: 0;
    flex: 1;
    min-width: 0;
  }

  /* 对话页：列宽受限并在主区水平居中，两侧留白 */
  .main-wrapper > .chat-home {
    width: min(100%, var(--chat-width-max));
    min-width: min(100%, var(--chat-width-min));
    max-width: var(--chat-width-max);
    align-self: center;
    margin-inline: auto;
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 100px 20px;
    background: var(--bg-primary);
    border-radius: 0;
    box-shadow: none;
  }

  .loading-spinner {
    width: 50px;
    height: 50px;
    border: 4px solid rgba(67, 97, 238, 0.1);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 20px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  /* 头部样式 */
  header {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px var(--shell-gutter);
    border-bottom: 1px solid var(--shell-divider);
    flex-shrink: 0;
    background: var(--bg-primary);
    min-height: calc(var(--header-control-h, 32px) + 20px);
  }
  
  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    transition: opacity 0.15s ease;
  }

  .logo:hover {
    transform: none;
    opacity: 0.85;
  }
  
  .logo-icon {
    width: var(--header-control-h, 32px);
    height: var(--header-control-h, 32px);
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    border-radius: var(--control-radius);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 14px;
    box-shadow: none;
  }
  
  .logo-text {
    font-size: 17px;
    font-weight: 650;
    letter-spacing: -0.02em;
    color: var(--text-primary);
  }
  
  .logo-text span {
    color: var(--primary);
  }

  /* 头部操作区域 */
  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  /* 主题切换按钮 */
  .theme-toggle {
    width: var(--header-control-h, 32px);
    height: var(--header-control-h, 32px);
    border-radius: var(--control-radius);
    border: 1px solid var(--shell-divider);
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
    box-shadow: none;
  }

  .theme-toggle:hover {
    transform: none;
    color: var(--text-primary);
    background: color-mix(in srgb, var(--text-secondary) 8%, transparent);
    box-shadow: none;
  }

  .settings-toggle {
    font-size: 13px;
  }

  .toolbox-toggle.active {
    color: var(--primary);
    border-color: color-mix(in srgb, var(--primary) 40%, var(--shell-divider));
    background: color-mix(in srgb, var(--primary) 10%, transparent);
  }

  .settings-toggle:hover {
    color: var(--primary, #4361ee);
  }

  /* 离线状态徽标 — 仅在 navigator.onLine 报告 false 时出现 */
  .offline-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: var(--control-radius);
    background: rgba(244, 67, 54, 0.1);
    color: #c62828;
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
  }
  .dark-mode .offline-badge {
    background: rgba(244, 67, 54, 0.22);
    color: #ff8a80;
  }
  .offline-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
    display: inline-block;
  }

  /* Spotlight 触发按钮 — 替代旧的搜索框，把 Cmd/Ctrl+K 提示做明显 */
  .spotlight-trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    min-width: 180px;
    height: var(--header-control-h, 32px);
    padding: 0 10px;
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid var(--shell-divider);
    border-radius: var(--control-radius);
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    box-shadow: none;
    transition: border-color 0.15s ease, color 0.15s ease, background 0.15s ease;
    box-sizing: border-box;
  }
  .spotlight-trigger:hover {
    border-color: color-mix(in srgb, var(--primary) 40%, var(--shell-divider));
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 5%, transparent);
    transform: none;
    box-shadow: none;
  }
  .spotlight-trigger:focus-visible {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.15);
  }
  .spotlight-trigger > i {
    font-size: 13px;
  }
  .spotlight-trigger-label {
    flex: 1;
    text-align: left;
    color: inherit;
    font-size: 12.5px;
  }
  .spotlight-trigger-kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 36px;
    height: 20px;
    padding: 0 6px;
    border-radius: 5px;
    background: color-mix(in srgb, var(--text-secondary) 8%, transparent);
    border: 1px solid var(--shell-divider);
    color: var(--text-secondary);
    font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.3px;
  }
  .spotlight-trigger:hover .spotlight-trigger-kbd {
    background: color-mix(in srgb, var(--primary) 10%, transparent);
    border-color: color-mix(in srgb, var(--primary) 28%, transparent);
    color: var(--primary);
  }

  /* 搜索区域样式 */
  .search-container {
    position: relative;
    width: 400px;
  }
  
  .search-container input {
    width: 100%;
    padding: 14px 20px 14px 50px;
    border-radius: 50px;
    border: 2px solid transparent;
    background: white;
    font-size: 16px;
    box-shadow: var(--shadow);
    transition: var(--transition);
  }
  
  .search-container input:focus {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 6px 25px rgba(67, 97, 238, 0.2);
    transform: translateY(-2px);
  }

  .search-container input:focus + .search-clear {
    opacity: 1;
  }

  .search-clear {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    width: 32px;
    height: 32px;
    border: none;
    background: rgba(67, 97, 238, 0.1);
    border-radius: 50%;
    color: var(--primary);
    font-size: 14px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.8;
    transition: var(--transition);
  }

  .search-clear:hover {
    background: rgba(67, 97, 238, 0.2);
    opacity: 1;
  }
  
  .search-icon {
    position: absolute;
    left: 20px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--gray);
    font-size: 18px;
  }
  
  /* 窄屏：单列并隐藏侧栏。断点须低于默认窗口宽，避免启动即乱版。
     用 .container > aside 提高优先级，压过 SideBar scoped 样式。 */
  @media (max-width: 900px) {
    .container {
      grid-template-columns: 1fr;
    }

    .container > aside,
    .sidebar-resizer {
      display: none;
    }

    .container .main-wrapper {
      grid-column: 1;
      grid-row: 2;
    }

    .featured-tools {
      grid-template-columns: 1fr;
    }
  }
  
  @media (max-width: 768px) {
    .search-container {
      width: 100%;
      margin-top: 20px;
    }
    
    header {
      flex-direction: column;
      align-items: flex-start;
      gap: 20px;
    }
  }
  
  /* Vue过渡效果 */
  .fade-enter-active {
    transition: opacity 0.3s ease, transform 0.3s ease;
  }
  
  .fade-leave-active {
    transition: opacity 0.2s ease, transform 0.2s ease;
  }
  
  .fade-enter-from {
    opacity: 0;
    transform: translateY(10px);
  }
  
  .fade-leave-to {
    opacity: 0;
    transform: translateY(-10px);
  }
</style>
