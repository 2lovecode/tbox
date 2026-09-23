import { computed, onBeforeUnmount, onMounted, ref, type Ref } from 'vue';

const STORAGE_KEY = 'tbox.sidebarWidthVw';

/** 相对视口的默认 / 上下限（设备自适应） */
const DEFAULT_VW = 15;
const MIN_VW = 12;
const MAX_VW = 24;
/** 绝对像素地板 / 天花板，避免极端分辨率下过窄或过宽 */
const MIN_PX = 180;
const MAX_PX = 300;

function readStoredVw(): number {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw == null) return DEFAULT_VW;
    const n = Number(raw);
    if (!Number.isFinite(n)) return DEFAULT_VW;
    return Math.min(MAX_VW, Math.max(MIN_VW, n));
  } catch {
    return DEFAULT_VW;
  }
}

function writeStoredVw(vw: number) {
  try {
    window.localStorage.setItem(STORAGE_KEY, String(vw));
  } catch {
    /* ignore quota */
  }
}

export function sidebarWidthBounds(viewportWidth: number): { min: number; max: number } {
  const w = Math.max(1, viewportWidth);
  const min = Math.max(MIN_PX, (MIN_VW / 100) * w);
  const max = Math.min(MAX_PX, (MAX_VW / 100) * w);
  return { min, max: Math.max(min, max) };
}

export function clampSidebarWidthPx(px: number, viewportWidth: number): number {
  const { min, max } = sidebarWidthBounds(viewportWidth);
  return Math.round(Math.min(max, Math.max(min, px)));
}

/**
 * 可拖拽侧栏宽度：以 vw 持久化，窗口变化时按视口重算并夹紧。
 */
export function useSidebarWidth(enabled: Ref<boolean>) {
  const preferredVw = ref(DEFAULT_VW);
  const widthPx = ref(200);
  const dragging = ref(false);

  function viewportWidth(): number {
    return window.innerWidth || document.documentElement.clientWidth || 1024;
  }

  function applyFromVw(vw: number) {
    const w = viewportWidth();
    const px = clampSidebarWidthPx((vw / 100) * w, w);
    widthPx.value = px;
    preferredVw.value = (px / w) * 100;
  }

  function refresh() {
    applyFromVw(preferredVw.value);
  }

  function setWidthFromClientX(clientX: number) {
    const w = viewportWidth();
    const px = clampSidebarWidthPx(clientX, w);
    widthPx.value = px;
    preferredVw.value = (px / w) * 100;
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging.value) return;
    setWidthFromClientX(e.clientX);
  }

  function onPointerUp(e: PointerEvent) {
    if (!dragging.value) return;
    dragging.value = false;
    try {
      (e.target as HTMLElement | null)?.releasePointerCapture?.(e.pointerId);
    } catch {
      /* ignore */
    }
    window.removeEventListener('pointermove', onPointerMove);
    window.removeEventListener('pointerup', onPointerUp);
    window.removeEventListener('pointercancel', onPointerUp);
    writeStoredVw(preferredVw.value);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }

  function startResize(e: PointerEvent) {
    if (!enabled.value) return;
    e.preventDefault();
    dragging.value = true;
    const el = e.currentTarget as HTMLElement | null;
    try {
      el?.setPointerCapture?.(e.pointerId);
    } catch {
      /* ignore */
    }
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    window.addEventListener('pointermove', onPointerMove);
    window.addEventListener('pointerup', onPointerUp);
    window.addEventListener('pointercancel', onPointerUp);
    // 宽度 = 指针到窗口左边；分隔条在侧栏右缘
    setWidthFromClientX(e.clientX);
  }

  function onWindowResize() {
    refresh();
  }

  onMounted(() => {
    preferredVw.value = readStoredVw();
    refresh();
    window.addEventListener('resize', onWindowResize);
  });

  onBeforeUnmount(() => {
    window.removeEventListener('resize', onWindowResize);
    window.removeEventListener('pointermove', onPointerMove);
    window.removeEventListener('pointerup', onPointerUp);
    window.removeEventListener('pointercancel', onPointerUp);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  });

  const cssWidth = computed(() => `${widthPx.value}px`);

  const ariaValue = computed(() => {
    const { min, max } = sidebarWidthBounds(viewportWidth());
    return {
      now: widthPx.value,
      min: Math.round(min),
      max: Math.round(max),
    };
  });

  return {
    widthPx,
    cssWidth,
    dragging,
    startResize,
    refresh,
    ariaValue,
  };
}
