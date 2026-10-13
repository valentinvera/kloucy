import { Text, View } from "react-native"
import type { NextAction } from "@/lib/next-actions"

interface NextActionsProps {
  actions: NextAction[]
}

export function NextActions({ actions }: NextActionsProps) {
  return (
    <View className="gap-3">
      <Text className="font-geist-semibold text-foreground text-lg">
        Next actions
      </Text>

      <View className="divide-y divide-border">
        {actions.map((action) => (
          <View className="gap-1 py-3" key={action.title}>
            <Text className="font-geist text-base text-foreground">
              {action.title}
            </Text>
            <Text className="font-geist text-muted text-sm">
              {action.detail}
            </Text>
          </View>
        ))}
      </View>
    </View>
  )
}
