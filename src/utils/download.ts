/**
 * Trigger a browser file download for `content` under `filename`. Works
 * inside Tauri WebView — the OS-native save dialog is intentionally
 * skipped so we don't need to wire up another plugin for tiny results.
 */
export function downloadTextFile(content: string, filename: string, mime = 'text/plain;charset=utf-8') {
  if (!content) return;
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.style.display = 'none';
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Defer revocation so Safari has time to start the download.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
