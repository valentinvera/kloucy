import { Button } from "heroui-native"
import { useCallback } from "react"
import { Text, View } from "react-native"

interface SessionRowProps {
  onRetry: (id: string) => void
  onStart: (id: string) => void
  onStop: (id: string) => void
  session: {
    id: string
    prompt: string | null
    status: string
  }
}

export function SessionRow({
  onRetry,
  onStart,
  onStop,
  session,
}: SessionRowProps) {
  const handleStart = useCallback(
    () => onStart(session.id),
    [onStart, session.id]
  )
  const handleStop = useCallback(() => onStop(session.id), [onStop, session.id])
  const handleRetry = useCallback(
    () => onRetry(session.id),
    [onRetry, session.id]
  )

  return (
    <View className="gap-2 py-3">
      <Text className="font-geist text-base text-foreground">
        {session.prompt ?? "Untitled session"}
      </Text>
      <Text className="font-geist text-muted text-xs">{session.status}</Text>
      <View className="flex-row gap-2">
        <Button onPress={handleStart} size="sm" variant="secondary">
          <Button.Label>Start</Button.Label>
        </Button>
        <Button onPress={handleStop} size="sm" variant="secondary">
          <Button.Label>Stop</Button.Label>
        </Button>
        <Button onPress={handleRetry} size="sm" variant="secondary">
          <Button.Label>Retry</Button.Label>
        </Button>
      </View>
    </View>
  )
}
