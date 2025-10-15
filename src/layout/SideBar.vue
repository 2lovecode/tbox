<script setup lang="ts">
import { Category } from '@/types/tools';
import { ref } from 'vue';
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
        <h3><i class="fas fa-list"></i> 工具分类</h3>
        <div class="categories">
          <div 
            v-for="category in fetchCategories()" 
            :key="category.id"
            class="category" 
            :class="{ active: store.activeCategory?.id === category.id }"
            @click="openCategory(category)"
          >
            <i :class="category.icon"></i>
            <span>{{ category.name }}</span>
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

  .tool-count {
    background: rgba(67, 97, 238, 0.1);
    color: var(--primary);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
    margin-left: 8px;
  }
</style>