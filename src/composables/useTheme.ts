import { ref, watch, onScopeDispose, type Ref } from "vue";

export type Theme = "light" | "dark" | "system";

const THEME_KEY = "app-theme";

/**
 * Theme composable. Each call creates an isolated, scoped theme state that
 * automatically tracks the system preference and cleans up listeners when
 * the consuming component is unmounted.
 */
export function useTheme() {
  const readInitial = (): Theme => {
    if (typeof window === "undefined") return "system";
    const stored = window.localStorage.getItem(THEME_KEY) as Theme | null;
    return stored ?? "system";
  };

  const theme = ref<Theme>(readInitial());
  const isDark = ref(false);

  const media = typeof window !== "undefined"
    ? window.matchMedia("(prefers-color-scheme: dark)")
    : null;

  const applyTheme = (value: Theme) => {
    if (value === "system") {
      isDark.value = media?.matches ?? false;
    } else {
      isDark.value = value === "dark";
    }

    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", isDark.value);
    }
  };

  applyTheme(theme.value);

  const stopWatch = watch(theme, (next) => {
    applyTheme(next);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(THEME_KEY, next);
    }
  });

  const handleSystemChange = () => {
    if (theme.value === "system") applyTheme("system");
  };
  media?.addEventListener("change", handleSystemChange);

  onScopeDispose(() => {
    stopWatch();
    media?.removeEventListener("change", handleSystemChange);
  });

  function setTheme(next: Theme) {
    theme.value = next;
  }

  function toggleTheme() {
    if (theme.value === "light") setTheme("dark");
    else if (theme.value === "dark") setTheme("system");
    else setTheme("light");
  }

  return { theme, isDark, setTheme, toggleTheme };
}

export type ThemeRef = Ref<Theme>;
