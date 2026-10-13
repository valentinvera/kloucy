import { useQuery } from "@tanstack/react-query"
import { Button } from "heroui-native"
import { useCallback, useState } from "react"
import { Text, View } from "react-native"
import { trpc } from "@/utils/trpc"

const VISIBLE_LOGS = 8

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
  const [isExpanded, setIsExpanded] = useState(false)

  const logs = useQuery(
    trpc.sessions.logs.queryOptions(
      { sessionId: session.id },
      { enabled: isExpanded, refetchInterval: 3000 }
    )
  )

  const handleStart = useCallback(
    () => onStart(session.id),
    [onStart, session.id]
  )
  const handleStop = useCallback(() => onStop(session.id), [onStop, session.id])
  const handleRetry = useCallback(
    () => onRetry(session.id),
    [onRetry, session.id]
  )
  const handleToggleOutput = useCallback(
    () => setIsExpanded((value) => !value),
    []
  )

  const visibleLogs = (logs.data ?? []).slice(-VISIBLE_LOGS)

  return (
    <View className="gap-2 py-3">
      <Text className="font-geist text-base text-foreground">
        {session.prompt ?? "Untitled session"}
      </Text>
      <Text className="font-geist text-muted text-xs">{session.status}</Text>

      {isExpanded ? (
        <View className="gap-1 rounded-md bg-muted p-3">
          {logs.isPending ? (
            <Text className="font-geist text-muted text-xs">
              Loading output…
            </Text>
          ) : null}

          {logs.error ? (
            <Text className="font-geist text-danger text-xs">
              {logs.error.message}
            </Text>
          ) : null}

          {logs.data && logs.data.length === 0 ? (
            <Text className="font-geist text-muted text-xs">
              No output yet.
            </Text>
          ) : null}

          {visibleLogs.map((log) => (
            <Text
              className={
                log.stream === "stderr"
                  ? "font-geist text-danger text-xs"
                  : "font-geist text-muted text-xs"
              }
              key={log.id}
            >
              {log.message}
            </Text>
          ))}
        </View>
      ) : null}

      <View className="flex-row flex-wrap gap-2">
        <Button onPress={handleStart} size="sm" variant="secondary">
          <Button.Label>Start</Button.Label>
        </Button>
        <Button onPress={handleStop} size="sm" variant="secondary">
          <Button.Label>Stop</Button.Label>
        </Button>
        <Button onPress={handleRetry} size="sm" variant="secondary">
          <Button.Label>Retry</Button.Label>
        </Button>
        <Button onPress={handleToggleOutput} size="sm" variant="ghost">
          <Button.Label>{isExpanded ? "Hide output" : "Output"}</Button.Label>
        </Button>
      </View>
    </View>
  )
}
