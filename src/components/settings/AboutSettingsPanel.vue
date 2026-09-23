<script setup lang="ts">
import { computed } from 'vue';
import { useToolStore } from '@/stores/tools';

const tools = useToolStore();
const toolCount = computed(() => tools.tools.length);
</script>

<template>
  <section class="about-panel">
    <h2>关于</h2>
    <p class="lead">TBox — 本地化开发者工具箱。</p>
    <dl class="meta">
      <div>
        <dt>版本</dt>
        <dd>3.0.0</dd>
      </div>
      <div>
        <dt>已收录工具</dt>
        <dd>{{ toolCount }} 个</dd>
      </div>
      <div>
        <dt>版权</dt>
        <dd>© 2025 TBox</dd>
      </div>
    </dl>
  </section>
</template>

<style scoped>
.about-panel h2 {
  margin: 0 0 8px;
  font-size: 16px;
  font-weight: 600;
}

.lead {
  margin: 0 0 16px;
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
  line-height: 1.6;
  max-width: 42em;
}

.meta {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 28em;
}

.meta > div {
  display: grid;
  grid-template-columns: 7em 1fr;
  gap: 8px;
  align-items: baseline;
}

.meta dt {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
}

.meta dd {
  margin: 0;
  font-size: 13px;
  color: var(--text-primary, #212529);
}
</style>
