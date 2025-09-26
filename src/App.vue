<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Tool, Category } from "@/types/tools";
import { useToolStore  }  from  "@/stores/tools";
import SideBar from "@/layout/SideBar.vue";
import HomePage from "@/views/HomePage.vue";




const store  = useToolStore()

// 工具数据
const tools = ref<Tool[]>([]);

// 加载categories
// 加载tools
onMounted(() => {
  invoke('get_categories').then((res) => {
    const fetchCategories = (res as Array<Category>).map((item: Category) => ({
      id: item.id,
      name: item.name,
      icon: "",
      count: item.count,
    }))
    store.setCategories(fetchCategories)
  })

  invoke('get_all_tools').then((res) => {
    const fetchTools = (res as Array<Tool>).map((item: Tool) => ({
      id: item.id,
      name: item.name,
      description: item.description,
      icon: item.icon,
      category: item.category,
      tags: item.tags,
      gradient: item.gradient,
    }))
    store.setTools(fetchTools)
  })
})

const searchQuery = ref('');
const searchTools = () => {
  // 实际项目中这里可以执行搜索逻辑
  console.log(`搜索工具: ${searchQuery.value}`);
};
</script>

<template>
      <div class="container">
        <header>
          <div class="logo">
            <div class="logo-icon">
              <i class="fas fa-toolbox"></i>
            </div>
            <div class="logo-text">万能<span>工具箱</span></div>
          </div>
          <div class="search-container">
            <i class="fas fa-search search-icon"></i>
            <input 
              type="text" 
              placeholder="搜索工具..." 
              v-model="searchQuery"
              @keyup.enter="searchTools"
            >
          </div>
        </header>
      <SideBar></SideBar>      
      <HomePage></HomePage>
      <footer>
        <p>© 2025 万能工具箱 | 版本: 3.0.0 | 已收录 {{ tools.length }} 个实用工具</p>
      </footer>
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
  }
  
  body {
    background: linear-gradient(135deg, #f5f7fa 0%, #e4edf5 100%);
    color: var(--dark);
    min-height: 100vh;
    padding: 20px;
  }
  
  .container {
    max-width: 1400px;
    margin: 0 auto;
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: 25px;
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
  
  /* 搜索区域样式 */
  .search-container {
    position: relative;
    width: 400px;
  }
  
  .search-container input {
    width: 100%;
    padding: 14px 20px 14px 50px;
    border-radius: 50px;
    border: none;
    background: white;
    font-size: 16px;
    box-shadow: var(--shadow);
    transition: var(--transition);
  }
  
  .search-container input:focus {
    outline: none;
    box-shadow: 0 6px 25px rgba(67, 97, 238, 0.2);
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
  .fade-enter-active,
  .fade-leave-active {
    transition: opacity 0.3s ease;
  }
  
  .fade-enter-from,
  .fade-leave-to {
    opacity: 0;
  }
</style>