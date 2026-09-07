<script setup lang="ts">
import { Tool, Category } from '@/types/tools';
import { useToolStore } from '@/stores/tools';
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRouter, useRoute } from 'vue-router';
import { useToast } from '@/composables/useToast';

const store = useToolStore();
const router = useRouter();
const route = useRoute();
const searchQuery = ref('');
const isLoading = ref(false);
const hoveredToolId = ref<number | null>(null);
const isCompactView = ref(false);
const searchResults = ref<Tool[]>([]);

const categoryCounts = computed<Record<number, number>>(() => {
  const counts: Record<number, number> = {};
  for (const tool of store.tools) {
    const catId = tool.category?.id;
    if (catId == null) continue;
    counts[catId] = (counts[catId] ?? 0) + 1;
  }
  return counts;
});

const categories = computed<Category[]>(() => {
  const total = Object.values(categoryCounts.value).reduce((sum, n) => sum + n, 0);
  return [
    { id: 0, name: '全部工具', icon: 'fas fa-star', count: total },
    ...store.categories.map((c) => ({ ...c, count: categoryCounts.value[c.id] ?? 0 })),
  ];
});

const openCategory = (category: Category) => {
  store.setActiveCategory(category);
};

// 计算属性 - 过滤后的工具列表
const filteredTools = computed(() => {
  let result = searchResults.value.length > 0 ? searchResults.value : store.tools;

  // 按分类过滤
  if (store.activeCategory && store.activeCategory.id !== 0) {
    result = result.filter(tool => tool.category?.id === store.activeCategory?.id);
  }
  return result;
});

// 监听搜索查询变化，使用Rust后端搜索
watch(searchQuery, async (newVal) => {
  if (newVal.trim()) {
    try {
      const results = await invoke<Tool[]>('search_tools', { query: newVal });
      searchResults.value = results;
    } catch (error) {
      console.error('Search failed:', error);
      // 降级到前端过滤
      searchResults.value = store.tools.filter(tool =>
        tool.name.toLowerCase().includes(newVal.toLowerCase()) ||
        tool.description.toLowerCase().includes(newVal.toLowerCase()) ||
        tool.tags.some(tag => tag.toLowerCase().includes(newVal.toLowerCase()))
      );
    }
  } else {
    searchResults.value = [];
  }
}, { immediate: true });

// 显示搜索结果状态
const showSearchResults = computed(() => searchQuery.value.trim().length > 0);

// 搜索结果统计
const searchResultCount = computed(() => filteredTools.value.length);

// 清空搜索
const clearSearch = () => {
  searchQuery.value = '';
  router.push({ path: '/toolbox' });
};

const routerMap: Record<number, string> = {
  1: 'image-compression',
  2: 'video-converter',
  3: 'password-manage',
  4: 'pdf-toolbox',
  5: 'screen-ruler',
  6: 'code-formatter',
  7: 'file-recovery',
  8: 'network-speed-test',
  9: 'json-tool',
  10: 'base64-tool',
  11: 'hash-generator',
  // 新增工具路由
  12: 'json-to-entity',
  13: 'json-diff',
  14: 'jwt-tool',
  15: 'regex-tester',
  16: 'timestamp-converter',
  17: 'http-request',
  18: 'text-tools',
  19: 'encoding-tools',
  // 更多新工具路由
  20: 'xml-tools',
  21: 'yaml-tools',
  22: 'gm-crypto',
  23: 'sql-tools',
  24: 'database-tools',
  25: 'image-tools',
  26: 'csv-tools',
  27: 'log-analyzer',
  28: 'color-tools',
  29: 'qrcode-tools',
  30: 'uuid-tools',
  31: 'cron-tools',
  32: 'number-tools',
  33: 'charset-tools',
  34: 'json-to-query',
  35: 'coordinate-tools',
  36: 'coordinate-visualizer'
}

const openTool = (tool: Tool) => {
  const toolRoute = routerMap[tool.id];
  if (toolRoute) {
    isNavigating.value = true;
    router.push({ path: `/${toolRoute}` });
    // 重置导航状态
    setTimeout(() => {
      isNavigating.value = false;
    }, 500);
  } else {
    toast.warning(`工具 "${tool.name}" 的路由尚未配置`);
  }
};

const setHoveredTool = (id: number | null) => {
  hoveredToolId.value = id;
}

// 监听搜索查询变化
watch(() => route.query.search, (newVal) => {
  if (newVal) {
    searchQuery.value = newVal as string;
  } else {
    searchQuery.value = '';
  }
}, { immediate: true });

// 监听本地搜索变化，同步到 URL（带防抖）
let searchTimeout: ReturnType<typeof setTimeout> | null = null;
watch(searchQuery, (newVal) => {
  // 如果正在导航中，不重复触发
  if (isNavigating.value) return;

  if (searchTimeout) clearTimeout(searchTimeout);

  searchTimeout = setTimeout(() => {
    if (newVal.trim()) {
      router.push({ path: '/toolbox', query: { search: newVal.trim() } });
    } else if (route.query.search) {
      router.push({ path: '/toolbox' });
    }
  }, 300);
});

// 标记是否正在导航，防止搜索watch干扰导航
const isNavigating = ref(false);
const toast = useToast();

</script>
<template>
    <main class="main-content">
        <div class="category-filters" v-if="!showSearchResults">
          <button
            v-for="category in categories"
            :key="category.id"
            type="button"
            class="category-chip"
            :class="{ active: (store.activeCategory?.id ?? 0) === category.id }"
            @click="openCategory(category)"
          >
            <i :class="category.icon"></i>
            <span>{{ category.name }}</span>
            <span class="chip-count">{{ category.count }}</span>
          </button>
        </div>

        <div class="toolbox-scroll">
        <!-- 搜索结果提示 -->
        <div v-if="showSearchResults" class="search-results-header">
          <div class="search-info">
            <i class="fas fa-search"></i>
            <span>搜索 "<strong>{{ searchQuery }}</strong>" 的结果</span>
            <span class="result-count">找到 {{ searchResultCount }} 个工具</span>
          </div>
          <button @click="clearSearch" class="clear-search-btn">
            <i class="fas fa-times"></i>
            清空搜索
          </button>
        </div>

        <div class="section-header" v-else>
            <h2 class="section-title">
            {{ store.activeCategory?.id === 0 ? '全部工具' : store.activeCategory?.name }}
            </h2>
            <div class="tool-count-badge" v-if="filteredTools.length > 0">
              共 {{ filteredTools.length }} 个工具
            </div>
        </div>

        <!-- 工具网格视图 -->
        <TransitionGroup
            v-if="!isLoading && filteredTools.length > 0"
            :name="showSearchResults ? '' : 'tool-card'"
            tag="div"
            class="tools-grid"
            :class="{ 'compact-view': isCompactView }"
        >
            <div
                v-for="tool in filteredTools"
                :key="showSearchResults ? `search-${tool.id}` : tool.id"
                class="tool-card"
                :class="{ 'hovered': hoveredToolId === tool.id, 'compact': isCompactView }"
                @click.stop="openTool(tool)"
                @mouseenter="setHoveredTool(tool.id)"
                @mouseleave="setHoveredTool(null)"
            >
                <div class="card-icon" :style="`background: ${tool.gradient};`">
                    <i :class="tool.icon"></i>
                </div>
                <div class="card-content">
                    <h3>{{ tool.name }}</h3>
                    <p v-if="!isCompactView" class="card-desc">{{ tool.description }}</p>
                    <div class="tool-tags" v-if="!isCompactView">
                        <span class="tag" v-for="tag in tool.tags.slice(0, 2)" :key="tag">{{ tag }}</span>
                    </div>
                </div>
            </div>
        </TransitionGroup>

        <div v-else-if="isLoading" class="loading-skeleton">
          <div v-for="i in 12" :key="i" class="skeleton-card" :class="{ compact: isCompactView }">
            <div class="skeleton-icon"></div>
            <div class="skeleton-content">
              <div class="skeleton-line"></div>
              <div class="skeleton-line short"></div>
            </div>
          </div>
        </div>

        <div v-else class="empty-state">
          <div class="empty-icon">
            <i class="fas" :class="showSearchResults ? 'fa-search-minus' : 'fa-search'"></i>
          </div>
          <h3>没有找到匹配的工具</h3>
          <p>
            {{ showSearchResults
              ? `尝试使用其他关键词搜索，或浏览全部 ${store.tools.length} 个工具`
              : '请尝试其他搜索词或选择不同的分类'
            }}
          </p>
          <button v-if="showSearchResults" @click="clearSearch" class="view-all-tools-btn">
            <i class="fas fa-th-large"></i>
            浏览全部工具
          </button>
        </div>

        <div class="view-toggle" v-if="!showSearchResults">
          <button
            class="toggle-btn"
            :class="{ active: !isCompactView }"
            @click="isCompactView = false"
            title="网格视图"
          >
            <i class="fas fa-th-large"></i>
          </button>
          <button
            class="toggle-btn"
            :class="{ active: isCompactView }"
            @click="isCompactView = true"
            title="紧凑视图"
          >
            <i class="fas fa-th"></i>
          </button>
        </div>
        </div>
    </main>
</template>
<style scoped> 
  /* 主内容区域 */
  .main-content {
    display: flex;
    flex-direction: column;
    gap: 20px;
    width: 100%;
    max-width: 100%;
    height: 100%;
    min-height: 0;
  }

  .toolbox-scroll {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 20px;
    overflow-y: auto;
    padding-right: 6px;
  }

  .category-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .category-chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid #e2e8f0;
    background: white;
    border-radius: 20px;
    color: #64748b;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .category-chip:hover {
    border-color: var(--primary);
    color: var(--primary);
  }

  .category-chip.active {
    background: rgba(67, 97, 238, 0.1);
    border-color: var(--primary);
    color: var(--primary);
    font-weight: 600;
  }

  .category-chip i {
    font-size: 12px;
  }

  .chip-count {
    background: #f1f5f9;
    color: #94a3b8;
    padding: 1px 7px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 500;
  }

  .category-chip.active .chip-count {
    background: var(--primary);
    color: white;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    font-size: 20px;
    font-weight: 700;
    color: var(--dark);
  }

  .tool-count-badge {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
    padding: 6px 14px;
    border-radius: 20px;
    font-size: 13px;
    font-weight: 500;
  }

  .section-header-right {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .view-toggle {
    display: flex;
    gap: 8px;
  }

  .toggle-btn {
    width: 36px;
    height: 36px;
    border: 1px solid #d9d9d9;
    background: white;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--gray);
    transition: all 0.2s;
  }

  .toggle-btn:hover {
    border-color: var(--primary);
    color: var(--primary);
  }

  .toggle-btn.active {
    background: var(--primary);
    border-color: var(--primary);
    color: white;
  }

  /* 工具网格 - 普通视图 */
  .tools-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 16px;
  }

  /* 工具网格 - 紧凑视图 */
  .tools-grid.compact-view {
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 12px;
  }

  /* 工具卡片样式 */
  .tool-card {
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    cursor: pointer;
    pointer-events: auto;
    overflow: hidden;
  }

  .tool-card * {
    pointer-events: auto;
  }

  .tool-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
  }

  /* 普通视图的卡片样式 */
  .tool-card .card-icon {
    width: 100%;
    height: 80px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tool-card .card-icon i {
    font-size: 32px;
    color: white;
  }

  .tool-card .card-content {
    padding: 14px 16px 16px;
  }

  .tool-card .card-content h3 {
    font-size: 14px;
    margin-bottom: 6px;
    color: var(--dark);
    font-weight: 600;
  }

  .tool-card .card-desc {
    color: var(--gray);
    font-size: 12px;
    line-height: 1.5;
    margin-bottom: 10px;
    flex: 1;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .tool-card .tool-tags {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  /* 紧凑视图的卡片样式 */
  .tool-card.compact {
    border-radius: 10px;
  }

  .tool-card.compact .card-icon {
    height: 56px;
  }

  .tool-card.compact .card-icon i {
    font-size: 24px;
  }

  .tool-card.compact .card-content {
    padding: 10px 12px 12px;
  }

  .tool-card.compact .card-content h3 {
    font-size: 13px;
    margin-bottom: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tool-card.compact:hover {
    transform: translateY(-3px);
  }

  .tool-card-enter-active {
    transition: all 0.3s ease;
  }

  .tool-card-enter-from {
    opacity: 0;
    transform: translateY(10px);
  }

  .tool-card-move {
    transition: transform 0.3s ease;
  }

  .tag {
    background: rgba(67, 97, 238, 0.08);
    color: var(--primary);
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
  }

  /* 搜索结果样式 */
  .search-results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    background: white;
    border-radius: 10px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  }

  .search-info {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--dark);
    font-size: 14px;
  }

  .search-info i {
    font-size: 18px;
    color: var(--primary);
  }

  .search-info strong {
    color: var(--primary);
    font-weight: 600;
  }

  .result-count {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
    padding: 4px 12px;
    border-radius: 20px;
    font-size: 13px;
    font-weight: 500;
    margin-left: 8px;
  }

  .clear-search-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px;
    border: 1px solid var(--primary);
    background: white;
    color: var(--primary);
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
  }

  .clear-search-btn:hover {
    background: var(--primary);
    color: white;
  }

  .view-all-tools-btn {
    margin-top: 15px;
    padding: 10px 20px;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    transition: var(--transition);
  }

  .view-all-tools-btn:hover {
    background: var(--secondary);
  }

  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 60px 20px;
    background: white;
    border-radius: 10px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  }

  .empty-icon {
    width: 80px;
    height: 80px;
    margin: 0 auto 20px;
    background: linear-gradient(135deg, rgba(67, 97, 238, 0.1), rgba(72, 149, 239, 0.1));
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-state i {
    font-size: 40px;
    color: var(--primary);
    opacity: 0.6;
  }

  .empty-state h3 {
    font-size: 20px;
    color: var(--dark);
    margin-bottom: 8px;
  }

  .empty-state p {
    color: var(--gray);
    font-size: 14px;
  }

  .loading-skeleton {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 16px;
  }

  .skeleton-card {
    background: white;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  }

  .skeleton-card.compact {
    border-radius: 10px;
  }

  .skeleton-icon {
    width: 100%;
    height: 80px;
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  .skeleton-card.compact .skeleton-icon {
    height: 56px;
  }

  .skeleton-content {
    padding: 14px 16px 16px;
  }

  .skeleton-line {
    height: 14px;
    background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
    background-size: 200% 100%;
    border-radius: 4px;
    margin-bottom: 8px;
    animation: skeleton-loading 1.5s ease-in-out infinite;
  }

  .skeleton-line.short {
    width: 60%;
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
