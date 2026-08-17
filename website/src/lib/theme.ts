import { useEffect, useState } from 'react'

export type Theme = 'dark' | 'light' | 'system'

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>('dark')

  useEffect(() => {
    try {
      const stored = localStorage.getItem('anastasia-theme') as Theme | null
      if (stored && ['dark', 'light', 'system'].includes(stored)) {
        setThemeState(stored)
      } else {
        setThemeState('dark')
      }
    } catch {
      setThemeState('dark')
    }
  }, [])

  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme)
    try {
      localStorage.setItem('anastasia-theme', newTheme)
    } catch {
      // Ignore localStorage errors
    }

    if (newTheme === 'dark') {
      document.documentElement.classList.add('dark')
    } else if (newTheme === 'light') {
      document.documentElement.classList.remove('dark')
    } else {
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      document.documentElement.classList.toggle('dark', isDark)
    }
  }

  const toggleTheme = () => {
    const isDark = document.documentElement.classList.contains('dark')
    setTheme(isDark ? 'light' : 'dark')
  }

  return { theme, setTheme, toggleTheme }
}
