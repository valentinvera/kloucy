import { Switch } from "heroui-native"
import { Text, View } from "react-native"

interface SafetyModeSwitchProps {
  isGuarded: boolean
  onChange: (guarded: boolean) => void
}

export function SafetyModeSwitch({
  isGuarded,
  onChange,
}: SafetyModeSwitchProps) {
  return (
    <View className="flex-row items-center gap-3 py-2">
      <Switch isSelected={isGuarded} onSelectedChange={onChange} />
      <Text className="font-geist text-muted text-sm">
        {isGuarded ? "Guarded: safe commands only" : "Open: any command"}
      </Text>
    </View>
  )
}
