<script setup lang="ts">
import { Tool } from '@/types/tools';
import { useToolStore } from '@/stores/tools';
import { ref, computed, onMounted } from 'vue';


const store = useToolStore();
const searchQuery = ref('');

// 计算属性 - 过滤后的工具列表
const filteredTools = computed(() => {
  let result = store.tools; 
  // 按分类过滤
  if (store.activeCategory && store.activeCategory.id !== 0) {
    result = result.filter(tool => tool.category?.id === store.activeCategory?.id);
  }
  
  // 按搜索词过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(tool => tool.name.toLowerCase().includes(query) || tool.description.toLowerCase().includes(query) || tool.tags.some(tag => tag.toLowerCase().includes(query)))
  }

  return result;
});

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

const openTool = (tool: Tool) => {
  console.log('openTool', tool);
  alert(`即将打开: ${tool.name}\n${tool.description}`);
};

</script>
<template>
    <main class="main-content">
        <div class="section-header">
            <h2 class="section-title">
            {{ store.activeCategory?.id === 0 ? '全部工具' : store.activeCategory?.name }}
            </h2>
            <a href="#" class="view-all">查看全部 <i class="fas fa-arrow-right"></i></a>
        </div>
        
        <div class="tools-grid">
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
</template>
<style scoped> 
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