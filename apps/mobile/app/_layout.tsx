import "@/styles/globals.css"
import { Geist_400Regular } from "@expo-google-fonts/geist/400Regular"
import { Geist_500Medium } from "@expo-google-fonts/geist/500Medium"
import { Geist_600SemiBold } from "@expo-google-fonts/geist/600SemiBold"
import { Geist_700Bold } from "@expo-google-fonts/geist/700Bold"
import { QueryClientProvider } from "@tanstack/react-query"
import { useFonts } from "expo-font"
import { Stack } from "expo-router"
import { hideAsync, preventAutoHideAsync } from "expo-splash-screen"
import { HeroUINativeProvider } from "heroui-native"
import { useEffect } from "react"
import { GestureHandlerRootView } from "react-native-gesture-handler"
import { KeyboardProvider } from "react-native-keyboard-controller"
import { AppThemeProvider } from "@/contexts/app-theme-context"
import { queryClient } from "@/utils/trpc"

preventAutoHideAsync().catch(() => undefined)

export default function Layout() {
  const [fontsLoaded, fontError] = useFonts({
    Geist_400Regular,
    Geist_500Medium,
    Geist_600SemiBold,
    Geist_700Bold,
  })

  useEffect(() => {
    if (fontsLoaded || fontError) {
      hideAsync().catch(() => undefined)
    }
  }, [fontError, fontsLoaded])

  if (!(fontsLoaded || fontError)) {
    return null
  }

  return (
    <QueryClientProvider client={queryClient}>
      <GestureHandlerRootView style={{ flex: 1 }}>
        <KeyboardProvider>
          <AppThemeProvider>
            <HeroUINativeProvider
              config={{ devInfo: { stylingPrinciples: false } }}
            >
              <Stack screenOptions={{ headerShown: false }} />
            </HeroUINativeProvider>
          </AppThemeProvider>
        </KeyboardProvider>
      </GestureHandlerRootView>
    </QueryClientProvider>
  )
}
