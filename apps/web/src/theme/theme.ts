export type Theme = 'light' | 'dark'

export const THEME_STORAGE_KEY = 'cc-theme'

export function resolveTheme(stored: string | null): Theme {
  if (stored === 'light' || stored === 'dark') return stored
  if (typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }
  return 'light'
}

export function readStoredTheme(): Theme {
  if (typeof window === 'undefined') return 'light'
  return resolveTheme(localStorage.getItem(THEME_STORAGE_KEY))
}

export function applyTheme(theme: Theme) {
  document.documentElement.dataset.theme = theme
  document.documentElement.style.colorScheme = theme
}

export function persistTheme(theme: Theme) {
  localStorage.setItem(THEME_STORAGE_KEY, theme)
}
