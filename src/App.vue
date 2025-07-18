<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Category {
  id: string;
  name: string;
  icon: string;
  toolCount: number;
}

interface Tool {
  id: number;
  name: string;
  description: string;
  icon: string;
  category?: string; // 可选属性
  tags: string[];
  gradient: string;
}

// 加载categories
// 加载tools
onMounted(() => {
  invoke('get_categories').then((res) => {
    console.log(res)
  })

  invoke('get_all_tools').then((res) => {
    console.log(res)
  })


})
const categories = ref([
  { id: 'all', name: '全部工具', icon: 'fas fa-star', toolCount: 24 },
  { id: 'system', name: '系统优化', icon: 'fas fa-laptop', toolCount: 5 },
  { id: 'file', name: '文件管理', icon: 'fas fa-file', toolCount: 6 },
  { id: 'image', name: '图片处理', icon: 'fas fa-image', toolCount: 4 },
  { id: 'video', name: '视频工具', icon: 'fas fa-video', toolCount: 3 },
  { id: 'network', name: '网络工具', icon: 'fas fa-network-wired', toolCount: 3 },
  { id: 'security', name: '安全工具', icon: 'fas fa-lock', toolCount: 2 },
  { id: 'dev', name: '开发工具', icon: 'fas fa-code', toolCount: 4 },
  { id: 'calc', name: '计算工具', icon: 'fas fa-calculator', toolCount: 3 }
]);

// 工具数据
const tools = ref([
  {
    id: 1,
    name: '图片压缩工具',
    description: '快速压缩图片大小而不损失质量，支持JPG、PNG等格式。',
    icon: 'fas fa-compress-arrows-alt',
    category: 'image',
    tags: ['图片处理', '优化'],
    gradient: 'linear-gradient(135deg, #4361ee, #4895ef)'
  },
  {
    id: 2,
    name: '视频格式转换',
    description: '支持多种视频格式转换，包括MP4, AVI, MOV等常见格式。',
    icon: 'fas fa-video',
    category: 'video',
    tags: ['视频处理', '转换'],
    gradient: 'linear-gradient(135deg, #f72585, #b5179e)'
  },
  {
    id: 3,
    name: '密码管理器',
    description: '安全存储和管理所有密码，自动生成强密码，保护您的账户安全。',
    icon: 'fas fa-key',
    category: 'security',
    tags: ['安全工具', '加密'],
    gradient: 'linear-gradient(135deg, #4cc9f0, #4895ef)'
  },
  {
    id: 4,
    name: 'PDF工具箱',
    description: '合并、分割、压缩PDF文件，添加水印和密码保护等实用功能。',
    icon: 'fas fa-file-pdf',
    category: 'file',
    tags: ['文件管理', 'PDF'],
    gradient: 'linear-gradient(135deg, #2ec4b6, #1a936f)'
  },
  {
    id: 5,
    name: '屏幕标尺',
    description: '在屏幕上测量元素尺寸，支持像素、厘米、英寸等多种单位。',
    icon: 'fas fa-ruler-combined',
    category: 'dev',
    tags: ['设计工具', '测量'],
    gradient: 'linear-gradient(135deg, #ff9e00, #ff5400)'
  },
  {
    id: 6,
    name: '代码格式化',
    description: '美化您的代码，支持多种编程语言，提高代码可读性和规范性。',
    icon: 'fas fa-code',
    category: 'dev',
    tags: ['开发工具', '编程'],
    gradient: 'linear-gradient(135deg, #7209b7, #560bad)'
  },
  {
    id: 7,
    name: '文件恢复工具',
    description: '恢复误删除的文件，支持多种文件系统和存储设备。',
    icon: 'fas fa-undo',
    category: 'file',
    tags: ['文件管理', '恢复'],
    gradient: 'linear-gradient(135deg, #3a0ca3, #4cc9f0)'
  },
  {
    id: 8,
    name: '网络测速',
    description: '测试您的网络下载、上传速度和延迟，提供详细分析报告。',
    icon: 'fas fa-wifi',
    category: 'network',
    tags: ['网络工具', '测速'],
    gradient: 'linear-gradient(135deg, #4361ee, #3a0ca3)'
  }
]);
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
const activeCategory = ref('all');
const searchQuery = ref('');

// 计算属性 - 过滤后的工具列表
const filteredTools = computed(() => {
  let result = tools.value;
  // 按分类过滤
  if (activeCategory.value !== 'all') {
    result = result.filter(tool => tool.category === activeCategory.value);
  }
  
  // 按搜索词过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(tool => tool.name.toLowerCase().includes(query) || tool.description.toLowerCase().includes(query) || tool.tags.some(tag => tag.toLowerCase().includes(query)))
  }

  return result;
});

// 方法
const setActiveCategory = (categoryId: string) => {
  activeCategory.value = categoryId;
  searchQuery.value = ''; // 切换分类时清空搜索
};

const openTool = (tool: Tool) => {
  alert(`即将打开: ${tool.name}\n${tool.description}`);
};

const searchTools = () => {
  // 实际项目中这里可以执行搜索逻辑
  console.log(`搜索工具: ${searchQuery.value}`);
};

const getCategoryName = (id: string) => {
  const category = categories.value.find(c => c.id === id);
  return category ? category.name : '';
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
            :class="{ active: activeCategory === category.id }"
            @click="setActiveCategory(category.id)"
          >
            <i :class="category.icon"></i>
            <span>{{ category.name }}</span>
            <span class="tool-count">{{ category.toolCount }}</span>
          </div>
        </div>
      </aside>
      
      <main class="main-content">
        <div class="section-header">
          <h2 class="section-title">
            {{ activeCategory === 'all' ? '全部工具' : getCategoryName(activeCategory) }}
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