import { createApp } from "vue";
import { createPinia } from "pinia";
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import {router} from "./router/main";

import App from "./App.vue";

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

const app = createApp(App)

app.use(router);
app.use(pinia);

// Global error handler — a single broken tool page shouldn't take the
// whole renderer down. Surface uncaught errors and unhandled promise
// rejections via the toast queue instead of silently crashing.
app.config.errorHandler = (err, _instance, info) => {
  console.error('[tbox] uncaught render error:', err, info);
  import('@/composables/useToast').then(({ useToast }) => {
    useToast().error('页面发生错误，请刷新或返回首页');
  });
};

if (typeof window !== 'undefined') {
  window.addEventListener('unhandledrejection', (event) => {
    const reason = event.reason;
    const msg = reason instanceof Error ? reason.message : String(reason ?? '未知错误');
    console.error('[tbox] unhandled rejection:', reason);
    import('@/composables/useToast').then(({ useToast }) => {
      useToast().error('操作失败：' + msg);
    });
  });
}

app.mount("#app");
