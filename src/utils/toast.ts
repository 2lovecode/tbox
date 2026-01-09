export const showToast = (message: string, type: 'success' | 'error' | 'info' | 'warning' = 'info', duration: number = 3000) => {
  if (typeof window !== 'undefined' && (window as any).$toast) {
    (window as any).$toast(message, type, duration)
  } else {
    // 降级到alert
    alert(message)
  }
}

export const toast = {
  success: (message: string) => showToast(message, 'success'),
  error: (message: string) => showToast(message, 'error'),
  info: (message: string) => showToast(message, 'info'),
  warning: (message: string) => showToast(message, 'warning'),
}
