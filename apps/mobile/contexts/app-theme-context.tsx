import {
  createContext,
  type PropsWithChildren,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react"
import { useColorScheme } from "react-native"
import { Uniwind } from "uniwind"

type AppTheme = "light" | "dark"

interface AppThemeContextValue {
  theme: AppTheme
  toggleTheme: () => void
}

const AppThemeContext = createContext<AppThemeContextValue | null>(null)

export function AppThemeProvider({ children }: PropsWithChildren) {
  const systemTheme = useColorScheme()
  const [selectedTheme, setSelectedTheme] = useState<AppTheme | null>(null)
  const theme = selectedTheme ?? (systemTheme === "dark" ? "dark" : "light")

  useEffect(() => {
    Uniwind.setTheme(theme)
  }, [theme])

  const value = useMemo<AppThemeContextValue>(
    () => ({
      theme,
      toggleTheme: () => {
        setSelectedTheme((currentTheme) => {
          const activeTheme =
            currentTheme ?? (systemTheme === "dark" ? "dark" : "light")

          return activeTheme === "dark" ? "light" : "dark"
        })
      },
    }),
    [systemTheme, theme]
  )

  return (
    <AppThemeContext.Provider value={value}>
      {children}
    </AppThemeContext.Provider>
  )
}

export function useAppTheme() {
  const context = useContext(AppThemeContext)

  if (!context) {
    throw new Error("useAppTheme must be used inside AppThemeProvider")
  }

  return context
}
