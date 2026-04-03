import { ref, watch } from "vue";

export type Theme = "light" | "dark" | "system";

const THEME_KEY = "app-theme";

const currentTheme = ref<Theme>(
  (localStorage.getItem(THEME_KEY) as Theme) || "system",
);
const isDark = ref(false);

// Update dark mode based on theme
function updateDarkMode() {
  if (currentTheme.value === "system") {
    isDark.value = window.matchMedia("(prefers-color-scheme: dark)").matches;
  } else {
    isDark.value = currentTheme.value === "dark";
  }

  // Apply to document
  if (isDark.value) {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}

// Initialize
updateDarkMode();

// Watch for theme changes
watch(currentTheme, updateDarkMode);

// Watch for system theme changes
window
  .matchMedia("(prefers-color-scheme: dark)")
  .addEventListener("change", () => {
    if (currentTheme.value === "system") {
      updateDarkMode();
    }
  });

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme;
    localStorage.setItem(THEME_KEY, theme);
  }

  function toggleTheme() {
    if (currentTheme.value === "light") {
      setTheme("dark");
    } else if (currentTheme.value === "dark") {
      setTheme("system");
    } else {
      setTheme("light");
    }
  }

  return {
    theme: currentTheme,
    isDark,
    setTheme,
    toggleTheme,
  };
}
