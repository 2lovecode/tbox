<script setup lang="ts">
import { Tool } from '@/types/tools';
import { useToolStore } from '@/stores/tools';
import { ref, computed, watch } from 'vue';
import { onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter, useRoute } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useRoleStore } from '@/stores/role';

const store = useToolStore();
const router = useRouter();
const route = useRoute();
const searchQuery = ref('');
const isLoading = ref(false);
const hoveredToolId = ref<number | null>(null);
const isCompactView = ref(false);
const searchResults = ref<Tool[]>([]);
const roleStore = useRoleStore();
const { selectedRoles: selectedRoleIds } = storeToRefs(roleStore);

// Story 1.5: 推荐工具缓存（按角色筛选）
const recommendedToolIds = ref<Set<number>>(new Set());
const roleNameByToolId = ref<Record<number, string[]>>({});
const roleFilterEnabled = ref(true);

// 计算属性 - 过滤后的工具列表
const filteredTools = computed(() => {
  let result = searchResults.value.length > 0 ? searchResults.value : store.tools;

  // Story 1.5: 角色过滤（仅在非搜索状态下生效，避免和搜索结果叠加）
  if (
    roleFilterEnabled.value &&
    selectedRoleIds.value.length > 0 &&
    recommendedToolIds.value.size > 0 &&
    searchResults.value.length === 0
  ) {
    result = result.filter((tool) => recommendedToolIds.value.has(tool.id));
  }

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

// Story 1.5: 角色相关的派生状态
const roleSummary = computed(() => {
  const ids = selectedRoleIds.value;
  if (ids.length === 0) return '';
  return roleStore.availableRoles
    .filter((r) => ids.includes(r.id))
    .map((r) => r.display_name)
    .join(' / ');
});

const showRoleFilterToggle = computed(
  () => selectedRoleIds.value.length > 0 && recommendedToolIds.value.size > 0
);

function toggleRoleFilter() {
  roleFilterEnabled.value = !roleFilterEnabled.value;
}

async function loadRecommendedTools() {
  const ids = selectedRoleIds.value;
  const idSet = new Set<number>();
  const tagMap: Record<number, string[]> = {};

  if (ids.length === 0) {
    recommendedToolIds.value = idSet;
    roleNameByToolId.value = tagMap;
    return;
  }

  await Promise.all(
    ids.map(async (roleId) => {
      try {
        const tools = await invoke<Tool[]>('get_tools_by_role', { roleId });
        const role = roleStore.availableRoles.find((r) => r.id === roleId);
        const roleName = role?.display_name ?? '';
        tools.forEach((t) => {
          idSet.add(t.id);
          if (roleName) {
            (tagMap[t.id] ??= []).push(roleName);
          }
        });
      } catch (error) {
        console.error(`Failed to load tools for role ${roleId}:`, error);
      }
    })
  );

  recommendedToolIds.value = idSet;
  roleNameByToolId.value = tagMap;
}

watch(
  () => [...selectedRoleIds.value],
  () => {
    loadRecommendedTools();
  },
  { deep: true }
);

// 清空搜索
const clearSearch = () => {
  searchQuery.value = '';
  router.push({ path: '/' });
};

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
    if ((window as any).$toast) {
      (window as any).$toast(`工具 "${tool.name}" 的路由尚未配置`, 'warning');
    } else {
      alert(`工具 "${tool.name}" 的路由尚未配置`);
    }
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
      router.push({ path: '/', query: { search: newVal.trim() } });
    } else if (route.query.search) {
      router.push({ path: '/' });
    }
  }, 300);
});

// 标记是否正在导航，防止搜索watch干扰导航
const isNavigating = ref(false);

onMounted(() => {
  loadRecommendedTools();
});

</script>
<template>
    <main class="main-content">
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
            <div class="section-header-right" v-if="roleSummary">
                <div class="role-summary">
                    <i class="fas fa-user-tag"></i>
                    <span>为你推荐：{{ roleSummary }}</span>
                </div>
                <button
                    v-if="showRoleFilterToggle"
                    class="role-filter-toggle"
                    :class="{ active: roleFilterEnabled }"
                    @click="toggleRoleFilter"
                    :title="roleFilterEnabled ? '点击查看全部工具' : '点击只看推荐工具'"
                >
                    <i :class="roleFilterEnabled ? 'fas fa-filter' : 'fas fa-filter-circle-xmark'"></i>
                    {{ roleFilterEnabled ? '只看推荐' : '查看全部' }}
                </button>
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
                    <div
                        v-if="!isCompactView && roleNameByToolId[tool.id]?.length"
                        class="role-tags"
                    >
                        <span
                            v-for="name in roleNameByToolId[tool.id]"
                            :key="name"
                            class="role-tag"
                        >
                            <i class="fas fa-user-tag"></i>
                            {{ name }}
                        </span>
                    </div>
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

        <!-- 视图切换和推荐工具 -->
        <template v-if="!showSearchResults">
            <div class="view-toggle">
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

            <div class="section-header featured-header" v-if="!showSearchResults">
                <h2 class="section-title">推荐工具</h2>
            </div>

            <div class="featured-tools" v-if="!showSearchResults">
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
        </template>
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

  .role-summary {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--gray);
    font-size: 13px;
    background: rgba(67, 97, 238, 0.06);
    border: 1px solid rgba(67, 97, 238, 0.15);
    padding: 6px 12px;
    border-radius: 999px;
  }

  .role-summary i {
    color: var(--primary);
  }

  .role-filter-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: white;
    border: 1px solid rgba(67, 97, 238, 0.3);
    color: var(--primary);
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .role-filter-toggle:hover {
    background: rgba(67, 97, 238, 0.08);
  }

  .role-filter-toggle.active {
    background: linear-gradient(135deg, #4361ee, #3f37c9);
    color: white;
    border-color: transparent;
    box-shadow: 0 4px 12px rgba(67, 97, 238, 0.25);
  }

  .role-filter-toggle:not(.active) {
    background: rgba(108, 117, 125, 0.08);
    color: var(--gray);
    border-color: rgba(108, 117, 125, 0.25);
  }

  .role-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin: 6px 0 4px;
  }

  .role-tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: linear-gradient(
      135deg,
      rgba(67, 97, 238, 0.1),
      rgba(63, 55, 201, 0.1)
    );
    color: var(--primary);
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 500;
    border: 1px solid rgba(67, 97, 238, 0.18);
  }

  .role-tag i {
    font-size: 10px;
  }

  /* 视图切换按钮 */
  .view-toggle {
    display: flex;
    gap: 8px;
    margin-bottom: 5px;
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

  /* 推荐工具区域 */
  .featured-header {
    margin-top: 10px;
  }

  .featured-tools {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 18px;
  }

  .featured-card {
    background: white;
    border-radius: 10px;
    display: flex;
    overflow: hidden;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
    transition: var(--transition);
    cursor: pointer;
  }

  .featured-card:hover {
    transform: translateY(-3px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
  }

  .featured-icon {
    width: 100px;
    background: linear-gradient(135deg, #4cc9f0, #4895ef);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
    color: white;
  }

  .featured-content {
    padding: 18px;
    flex: 1;
  }

  .featured-content h3 {
    font-size: 17px;
    margin-bottom: 6px;
    color: var(--dark);
  }

  .featured-content p {
    color: var(--gray);
    font-size: 13px;
    line-height: 1.5;
    margin-bottom: 12px;
  }

  .featured-btn {
    background: var(--primary);
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
  }

  .featured-btn:hover {
    background: var(--secondary);
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
