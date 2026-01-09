<script setup lang="ts">
import { useRouter } from 'vue-router'

const props = defineProps<{
  title: string
  showBack?: boolean
}>()

const router = useRouter()

const goBack = () => {
  router.push('/')
}
</script>

<template>
  <div class="page-header">
    <div class="header-content">
      <div class="header-left">
        <button v-if="showBack" @click="goBack" class="back-btn">
          <i class="fas fa-arrow-left"></i>
        </button>
        <h1>{{ title }}</h1>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page-header {
  margin-bottom: 30px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 15px;
}

.back-btn {
  width: 40px;
  height: 40px;
  border: none;
  background: white;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--gray);
  transition: var(--transition);
  box-shadow: var(--shadow);
}

.back-btn:hover {
  background: var(--primary);
  color: white;
  transform: translateX(-3px);
}

.page-header h1 {
  font-size: 28px;
  font-weight: 700;
  color: var(--dark);
  margin: 0;
}
</style>
