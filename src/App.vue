<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
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
import { useTheme } from "@/composables/useTheme";
import { useKeyboardShortcuts } from "@/composables/useKeyboardShortcuts";
import { formatShortcut } from "@/composables/useKeyboardShortcuts";

const store  = useToolStore()
const searchStore = useSearchStore()
const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()
const shortcuts = useKeyboardShortcuts()

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
        icon: "",
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

// 计算是否显示侧边栏（在工具页面不显示）
const showSidebar = computed(() => route.path === '/')

// Detect platform once for the keyboard-shortcut hint in the header.
const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);
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
      <div class="container" :class="{ 'no-sidebar': !showSidebar, 'dark-mode': isDark }">
        <header>
          <div class="logo" @click="$router.push('/')" style="cursor: pointer;">
            <div class="logo-icon">
              <i class="fas fa-toolbox"></i>
            </div>
            <div class="logo-text">万能<span>工具箱</span></div>
          </div>
          <div class="header-actions">
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
            <button @click="toggleTheme" class="theme-toggle" :title="isDark ? '切换到浅色模式' : '切换到深色模式'">
              <i :class="isDark ? 'fas fa-sun' : 'fas fa-moon'"></i>
            </button>
          </div>
        </header>
      <SideBar v-if="showSidebar"></SideBar>      
      <div class="main-wrapper">
        <Transition name="fade" mode="out-in">
          <RouterView v-if="!isLoading" />
          <div v-else class="loading-container">
            <div class="loading-spinner"></div>
            <p>加载中...</p>
          </div>
        </Transition>
      </div>
      <footer>
        <p>© 2025 万能工具箱 | 版本: 3.0.0 | 已收录 {{ tools.length }} 个实用工具</p>
        </footer>
        <Toast />
        <SpotlightSearch />
        <ShortcutHints />
      </div>
</template>

<style>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
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
    --text-primary: #212529;
    --text-secondary: #6c757d;
    --border-color: rgba(0, 0, 0, 0.1);
  }

  .dark-mode {
    --bg-primary: #1a1a2e;
    --bg-secondary: #16213e;
    --bg-tertiary: #0f3460;
    --text-primary: #e4e4e7;
    --text-secondary: #a1a1aa;
    --border-color: rgba(255, 255, 255, 0.1);
    --shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  body {
    background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
    color: var(--text-primary);
    min-height: 100vh;
    padding: 20px;
    transition: background 0.3s ease, color 0.3s ease;
  }
  
  .container {
    max-width: 1400px;
    margin: 0 auto;
    display: grid;
    grid-template-columns: 210px 1fr;
    gap: 20px;
  }

  .container.no-sidebar {
    grid-template-columns: 1fr;
  }

  .container.no-sidebar .main-wrapper {
    grid-column: 1;
    max-width: 100%;
  }

  .main-wrapper {
    min-height: 400px;
    background: transparent;
  }

  .container.no-sidebar .main-wrapper {
    grid-column: 1 / -1;
    width: 100%;
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 100px 20px;
    background: white;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
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
    padding: 20px 0;
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
  }
  
  .logo {
    display: flex;
    align-items: center;
    gap: 15px;
    transition: var(--transition);
  }

  .logo:hover {
    transform: scale(1.02);
  }
  
  .logo-icon {
    width: 48px;
    height: 48px;
    background: linear-gradient(135deg, var(--primary), var(--secondary));
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-size: 24px;
    box-shadow: var(--shadow);
  }
  
  .logo-text {
    font-size: 28px;
    font-weight: 700;
    color: var(--dark);
  }
  
  .logo-text span {
    color: var(--primary);
  }

  /* 头部操作区域 */
  .header-actions {
    display: flex;
    align-items: center;
    gap: 20px;
  }

  /* 主题切换按钮 */
  .theme-toggle {
    width: 45px;
    height: 45px;
    border-radius: 12px;
    border: none;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-size: 18px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
    box-shadow: var(--shadow);
  }

  .theme-toggle:hover {
    transform: scale(1.1);
    box-shadow: 0 6px 25px rgba(67, 97, 238, 0.3);
  }

  /* Spotlight 触发按钮 — 替代旧的搜索框，把 Cmd/Ctrl+K 提示做明显 */
  .spotlight-trigger {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    min-width: 240px;
    padding: 10px 16px;
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    border-radius: 999px;
    cursor: pointer;
    font-family: inherit;
    font-size: 14px;
    box-shadow: var(--shadow);
    transition: var(--transition);
  }
  .spotlight-trigger:hover {
    border-color: var(--primary);
    color: var(--primary);
    transform: translateY(-1px);
    box-shadow: 0 6px 25px rgba(67, 97, 238, 0.2);
  }
  .spotlight-trigger:focus-visible {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.2);
  }
  .spotlight-trigger > i {
    font-size: 15px;
  }
  .spotlight-trigger-label {
    flex: 1;
    text-align: left;
    color: var(--text-secondary);
    font-size: 15px;
  }
  .spotlight-trigger-kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 48px;
    height: 28px;
    padding: 0 10px;
    border-radius: 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.5px;
  }
  .spotlight-trigger:hover .spotlight-trigger-kbd {
    background: rgba(67, 97, 238, 0.12);
    border-color: rgba(67, 97, 238, 0.35);
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
  
  /* 底部信息 */
  footer {
    grid-column: 1 / -1;
    text-align: center;
    padding: 30px 0;
    color: var(--gray);
    font-size: 14px;
    border-top: 1px solid rgba(0, 0, 0, 0.05);
    margin-top: 20px;
  }
  
  /* 响应式设计 */
  @media (max-width: 1100px) {
    .container {
      grid-template-columns: 1fr;
    }
    
    .sidebar {
      display: none;
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
