<script setup lang="ts">
import { Tool } from '@/types/tools';
import { useToolStore } from '@/stores/tools';
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';

const store = useToolStore();
const router = useRouter();
const route = useRoute();
const searchQuery = ref('');
const isLoading = ref(false);

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

const routerMap: Record<number, string> = {
  1: 'image-compression',
  2: 'video-converter',
  3: 'password-manage',
  4: 'pdf-toolbox',
  5: 'screen-ruler',
  6: 'code-formatter',
  7: 'file-recovery',
  8: 'network-speed-test',
  9: 'json-tool'
}

const openTool = (tool: Tool) => {
  const toolRoute = routerMap[tool.id];
  if (toolRoute) {
    router.push({ path: `/${toolRoute}` });
  } else {
    if ((window as any).$toast) {
      (window as any).$toast(`工具 "${tool.name}" 的路由尚未配置`, 'warning');
    } else {
      alert(`工具 "${tool.name}" 的路由尚未配置`);
    }
  }
};

// 监听搜索查询变化
watch(() => route.query.search, (newVal) => {
  if (newVal) {
    searchQuery.value = newVal as string;
  }
}, { immediate: true });

</script>
<template>
    <main class="main-content">
        <div class="section-header">
            <h2 class="section-title">
            {{ store.activeCategory?.id === 0 ? '全部工具' : store.activeCategory?.name }}
            </h2>
            <div class="tool-count-badge" v-if="filteredTools.length > 0">
              共 {{ filteredTools.length }} 个工具
            </div>
        </div>
        
        <TransitionGroup 
            v-if="!isLoading && filteredTools.length > 0"
            name="tool-card" 
            tag="div" 
            class="tools-grid"
        >
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
        </TransitionGroup>
        
        <div v-else-if="isLoading" class="loading-skeleton">
          <div v-for="i in 6" :key="i" class="skeleton-card">
            <div class="skeleton-header"></div>
            <div class="skeleton-content">
              <div class="skeleton-line"></div>
              <div class="skeleton-line short"></div>
              <div class="skeleton-tags">
                <div class="skeleton-tag"></div>
                <div class="skeleton-tag"></div>
              </div>
            </div>
          </div>
        </div>
            
        <div v-else class="empty-state">
          <div class="empty-icon">
            <i class="fas fa-search"></i>
          </div>
          <h3>没有找到匹配的工具</h3>
          <p>请尝试其他搜索词或选择不同的分类</p>
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
    width: 100%;
    max-width: 100%;
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
  
  .tool-count-badge {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 14px;
    font-weight: 500;
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
    position: relative;
  }
  
  .tool-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, rgba(67, 97, 238, 0.05), rgba(72, 149, 239, 0.05));
    opacity: 0;
    transition: var(--transition);
  }
  
  .tool-card:hover {
    transform: translateY(-8px) scale(1.02);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.15);
  }
  
  .tool-card:hover::before {
    opacity: 1;
  }
  
  .tool-card-enter-active {
    transition: all 0.4s ease;
  }
  
  .tool-card-enter-from {
    opacity: 0;
    transform: translateY(20px) scale(0.9);
  }
  
  .tool-card-move {
    transition: transform 0.4s ease;
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
    transition: var(--transition);
    cursor: pointer;
  }

  .featured-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.15);
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
    padding: 80px 20px;
    background: white;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
  }
  
  .empty-icon {
    width: 100px;
    height: 100px;
    margin: 0 auto 30px;
    background: linear-gradient(135deg, rgba(67, 97, 238, 0.1), rgba(72, 149, 239, 0.1));
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .empty-state i {
    font-size: 48px;
    color: var(--primary);
    opacity: 0.6;
  }
  
  .empty-state h3 {
    font-size: 24px;
    color: var(--dark);
    margin-bottom: 10px;
  }
  
  .empty-state p {
    color: var(--gray);
    font-size: 16px;
  }

  .loading-skeleton {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 25px;
  }

  .skeleton-card {
    background: white;
    border-radius: var(--border-radius);
    overflow: hidden;
    box-shadow: var(--shadow);
  }

  .skeleton-header {
    height: 120px;
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  .skeleton-content {
    padding: 20px;
  }

  .skeleton-line {
    height: 16px;
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    border-radius: 4px;
    margin-bottom: 10px;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  .skeleton-line.short {
    width: 60%;
  }

  .skeleton-tags {
    display: flex;
    gap: 8px;
    margin-top: 15px;
  }

  .skeleton-tag {
    width: 60px;
    height: 24px;
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    border-radius: 12px;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  @keyframes skeleton-loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style>