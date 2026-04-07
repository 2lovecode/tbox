<script setup lang="ts">
import { Category } from '@/types/tools';
import { useToolStore } from '@/stores/tools';
import { useRouter } from 'vue-router';

const store = useToolStore()
const router = useRouter()

const fetchCategories = (): Category[] => {
    const remoteCategories = store.categories
    return [
      { id: 0, name: '全部工具', icon: 'fas fa-star', count: remoteCategories.reduce((sum, cat) => sum + cat.count, 0) },
      ...remoteCategories
    ]
}

const openCategory = (category: Category) => {
  store.setActiveCategory(category);
  router.push({ path: "/" })
}


</script>
<template>
    <aside class="sidebar">
        <h3><i class="fas fa-th-large"></i> 工具分类</h3>
        <div class="categories">
          <div
            v-for="category in fetchCategories()"
            :key="category.id"
            class="category"
            :class="{ active: store.activeCategory?.id === category.id }"
            @click="openCategory(category)"
          >
            <div class="category-left">
              <i :class="category.icon"></i>
              <span class="category-name">{{ category.name }}</span>
            </div>
            <span class="tool-count">{{ category.count }}</span>
          </div>
        </div>
    </aside>
</template>


<style scoped>
  /* 侧边栏样式 */
  .sidebar {
    background: white;
    border-radius: var(--border-radius);
    padding: 20px;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
    height: fit-content;
  }

  .sidebar h3 {
    margin-bottom: 16px;
    font-size: 13px;
    color: #94a3b8;
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .sidebar h3 i {
    color: var(--primary);
    font-size: 12px;
  }

  .categories {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .category {
    padding: 10px 12px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
    transition: all 0.2s ease;
    color: #64748b;
    font-size: 14px;
    position: relative;
  }

  .category-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .category:hover {
    background: #f1f5f9;
    color: var(--primary);
  }

  .category.active {
    background: linear-gradient(135deg, rgba(67, 97, 238, 0.1), rgba(67, 97, 238, 0.05));
    color: var(--primary);
    font-weight: 600;
  }

  .category.active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 24px;
    background: var(--primary);
    border-radius: 0 4px 4px 0;
  }

  .category i {
    width: 18px;
    text-align: center;
    font-size: 15px;
    flex-shrink: 0;
  }

  .category-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tool-count {
    background: #f1f5f9;
    color: #94a3b8;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
    flex-shrink: 0;
    min-width: 28px;
    text-align: center;
    transition: all 0.2s ease;
  }

  .category:hover .tool-count {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
  }

  .category.active .tool-count {
    background: var(--primary);
    color: white;
  }
</style>