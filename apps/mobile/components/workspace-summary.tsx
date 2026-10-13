import { Text, View } from "react-native"
import { type SessionCounts, summarizeCounts } from "@/lib/session-counts"

interface WorkspaceSummaryProps {
  counts: SessionCounts
  localPath?: string
  name: string
}

export function WorkspaceSummary({
  counts,
  localPath,
  name,
}: WorkspaceSummaryProps) {
  return (
    <View className="gap-1">
      <Text className="font-geist-semibold text-2xl text-foreground">
        {name}
      </Text>
      {localPath ? (
        <Text className="font-geist text-muted text-xs">{localPath}</Text>
      ) : null}
      <Text className="font-geist text-foreground text-sm">
        {summarizeCounts(counts)}
      </Text>
    </View>
  )
}
