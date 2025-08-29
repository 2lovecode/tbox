<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Category {
  id: number;
  name: string;
  icon: string;
  count: number;
}

interface Tool {
  id: number;
  name: string;
  description: string;
  icon: string;
  category?: Category; // 可选属性
  tags: string[];
  gradient: string;
}

const categories = ref<Category[]>([]);

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
   categories.value = [
      { id: 0, name: '全部工具', icon: 'fas fa-star', count: fetchCategories.reduce((sum, cat) => sum + cat.count, 0) },
      ...fetchCategories
    ];
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
    tools.value = fetchTools
  })


})

// 推荐工具数据
const featuredTools = ref([
  {
    id: 101,
    name: '系统清理大师',
    description: '一键清理系统垃圾文件、注册表错误和无效快捷方式，释放磁盘空间，提升系统运行速度。',
    icon: 'fas fa-sync-alt',
    tags: ['系统优化', '清理'],
    gradient: 'linear-gradient(135deg, #4cc9f0, #4895ef)'
  },
  {
    id: 102,
    name: '隐私保护工具',
    description: '深度清理浏览痕迹、临时文件和隐私数据，保护您的个人隐私不被泄露。',
    icon: 'fas fa-shield-alt',
    tags: ['安全工具', '隐私'],
    gradient: 'linear-gradient(135deg, #f72585, #b5179e)'
  }
]);
const activeCategory = ref<Category>();
const searchQuery = ref('');

// 计算属性 - 过滤后的工具列表
const filteredTools = computed(() => {
  let result = tools.value;
  // 按分类过滤
  if (activeCategory.value && activeCategory.value.id !== 0) {
    result = result.filter(tool => tool.category?.id === activeCategory.value?.id);
  }
  
  // 按搜索词过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(tool => tool.name.toLowerCase().includes(query) || tool.description.toLowerCase().includes(query) || tool.tags.some(tag => tag.toLowerCase().includes(query)))
  }

  return result;
});

// 方法
const setActiveCategory = (category: Category) => {
  activeCategory.value = category;
  searchQuery.value = ''; // 切换分类时清空搜索
};

const openTool = (tool: Tool) => {
  console.log('openTool', tool);
  alert(`即将打开: ${tool.name}\n${tool.description}`);
};

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
      
      <aside class="sidebar">
        <h3><i class="fas fa-list"></i> 工具分类</h3>
        <div class="categories">
          <div 
            v-for="category in categories" 
            :key="category.id"
            class="category" 
            :class="{ active: activeCategory?.id === category.id }"
            @click="setActiveCategory(category)"
          >
            <i :class="category.icon"></i>
            <span>{{ category.name }}</span>
            <span class="tool-count">{{ category.count }}</span>
          </div>
        </div>
      </aside>
      
      <main class="main-content">
        <div class="section-header">
          <h2 class="section-title">
            {{ activeCategory?.id === 0 ? '全部工具' : activeCategory?.name }}
          </h2>
          <a href="#" class="view-all">查看全部 <i class="fas fa-arrow-right"></i></a>
        </div>
        
        <div class="tools-grid">
          <transition-group name="fade">
            <div 
              v-for="tool in filteredTools" 
              :key="tool.id"
              class="tool-card"
              @click="openTool(tool)"
            >
              <div class="card-header" :style="`background: ${tool.gradient};`">
                <i :class="tool.icon"></i>
              </div>
              <div class="card-content">
                <h3>{{ tool.name }}</h3>
                <p>{{ tool.description }}</p>
                <div class="tool-tags">
                  <span class="tag" v-for="tag in tool.tags" :key="tag">{{ tag }}</span>
                </div>
              </div>
            </div>
          </transition-group>
          
          <div v-if="filteredTools.length === 0" class="empty-state">
            <i class="fas fa-search"></i>
            <p>没有找到匹配的工具，请尝试其他搜索词</p>
          </div>
        </div>
        
        <div class="section-header">
          <h2 class="section-title">推荐工具</h2>
        </div>
        
        <div class="featured-tools">
          <div 
            v-for="featured in featuredTools" 
            :key="featured.id"
            class="featured-card"
          >
            <div class="featured-icon" :style="`background: ${featured.gradient};`">
              <i :class="featured.icon"></i>
            </div>
            <div class="featured-content">
              <h3>{{ featured.name }}</h3>
              <p>{{ featured.description }}</p>
              <button class="featured-btn" @click="openTool(featured)">立即使用</button>
            </div>
          </div>
        </div>
      </main>
      
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
  
  /* 侧边栏样式 */
  .sidebar {
    background: white;
    border-radius: var(--border-radius);
    padding: 25px;
    box-shadow: var(--shadow);
    height: fit-content;
  }
  
  .sidebar h3 {
    margin-bottom: 20px;
    font-size: 18px;
    color: var(--dark);
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .sidebar h3 i {
    color: var(--primary);
  }
  
  .categories {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  
  .category {
    padding: 12px 15px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    transition: var(--transition);
    color: var(--gray);
    font-weight: 500;
  }
  
  .category:hover {
    background: rgba(67, 97, 238, 0.05);
    color: var(--primary);
  }
  
  .category.active {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
  }
  
  .category i {
    width: 20px;
    text-align: center;
    font-size: 18px;
  }
  
  /* 主内容区域 */
  .main-content {
    display: flex;
    flex-direction: column;
    gap: 25px;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  
  .section-title {
    font-size: 24px;
    font-weight: 700;
    color: var(--dark);
  }
  
  .view-all {
    color: var(--primary);
    font-weight: 500;
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .tools-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 25px;
  }
  
  /* 工具卡片样式 */
  .tool-card {
    background: white;
    border-radius: var(--border-radius);
    overflow: hidden;
    box-shadow: var(--shadow);
    transition: var(--transition);
    cursor: pointer;
  }
  
  .tool-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.1);
  }
  
  .card-header {
    height: 120px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--primary), var(--accent));
  }
  
  .card-header i {
    font-size: 48px;
    color: white;
  }
  
  .card-content {
    padding: 20px;
  }
  
  .card-content h3 {
    font-size: 18px;
    margin-bottom: 10px;
    color: var(--dark);
  }
  
  .card-content p {
    color: var(--gray);
    font-size: 14px;
    line-height: 1.6;
    margin-bottom: 15px;
  }
  
  .tool-tags {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  
  .tag {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
    padding: 4px 10px;
    border-radius: 50px;
    font-size: 12px;
    font-weight: 500;
  }
  
  /* 推荐工具区域 */
  .featured-tools {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 25px;
  }
  
  .featured-card {
    background: white;
    border-radius: var(--border-radius);
    display: flex;
    overflow: hidden;
    box-shadow: var(--shadow);
  }
  
  .featured-icon {
    width: 120px;
    background: linear-gradient(135deg, #4cc9f0, #4895ef);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 36px;
    color: white;
  }
  
  .featured-content {
    padding: 25px;
    flex: 1;
  }
  
  .featured-content h3 {
    font-size: 20px;
    margin-bottom: 10px;
    color: var(--dark);
  }
  
  .featured-content p {
    color: var(--gray);
    line-height: 1.6;
    margin-bottom: 15px;
  }
  
  .featured-btn {
    background: var(--primary);
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
  }
  
  .featured-btn:hover {
    background: var(--secondary);
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
  
  .tool-count {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
    margin-left: 8px;
  }
  
  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 50px 0;
  }
  
  .empty-state i {
    font-size: 48px;
    color: var(--gray);
    opacity: 0.5;
    margin-bottom: 20px;
  }
  
  .empty-state p {
    color: var(--gray);
    font-size: 16px;
  }
</style>