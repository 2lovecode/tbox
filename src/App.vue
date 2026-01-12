<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Tool, Category } from "@/types/tools";
import { useToolStore  }  from  "@/stores/tools";
import SideBar from "@/layout/SideBar.vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import Toast from "@/components/Toast.vue";
import { useTheme } from "@/composables/useTheme";

const store  = useToolStore()
const route = useRoute()
const router = useRouter()
const { isDark, toggleTheme } = useTheme()

// 工具数据
const tools = ref<Tool[]>([]);
const isLoading = ref(true);

// 加载categories
// 加载tools
onMounted(async () => {
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

const searchQuery = ref('');

// 监听路由变化，同步搜索框状态
watch(() => route.query.search, (newVal) => {
  searchQuery.value = (newVal as string) || '';
}, { immediate: true });

// 搜索功能 - 实时搜索并跳转到搜索结果页
const searchTools = () => {
  if (searchQuery.value.trim()) {
    // 跳转到首页并传递搜索参数
    router.push({ path: '/', query: { search: searchQuery.value.trim() } });
  } else if (route.query.search) {
    // 清空搜索时移除搜索参数
    router.push({ path: '/' });
  }
};

// 清空搜索
const clearSearch = () => {
  searchQuery.value = '';
  if (route.query.search) {
    router.push({ path: '/' });
  }
};

// 计算是否显示侧边栏（在工具页面不显示）
const showSidebar = computed(() => route.path === '/')
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
            <div class="search-container" v-if="showSidebar">
              <i class="fas fa-search search-icon"></i>
              <input
                type="text"
                placeholder="搜索工具名称、描述或标签..."
                v-model="searchQuery"
                @keyup.enter="searchTools"
                @input="searchTools"
              >
              <button
                v-if="searchQuery"
                @click="clearSearch"
                class="search-clear"
                title="清空搜索"
              >
                <i class="fas fa-times"></i>
              </button>
            </div>
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
    grid-template-columns: 260px 1fr;
    gap: 25px;
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