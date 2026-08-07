import { Ionicons } from "@expo/vector-icons"
import { Switch, useThemeColor } from "heroui-native"
import { View } from "react-native"

import { useAppTheme } from "@/contexts/app-theme-context"

export function ThemeToggle() {
  const { theme, toggleTheme } = useAppTheme()
  const foregroundColor = useThemeColor("foreground")

  return (
    <View className="px-4">
      <Switch isSelected={theme === "dark"} onSelectedChange={toggleTheme}>
        <Switch.StartContent>
          <Ionicons color={foregroundColor} name="sunny-outline" size={14} />
        </Switch.StartContent>
        <Switch.EndContent>
          <Ionicons color={foregroundColor} name="moon-outline" size={14} />
        </Switch.EndContent>
      </Switch>
    </View>
  )
}
