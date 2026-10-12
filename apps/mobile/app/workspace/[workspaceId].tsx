import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useLocalSearchParams, useRouter } from "expo-router"
import { Button, Spinner } from "heroui-native"
import { useCallback } from "react"
import { Text, View } from "react-native"
import { SessionRow } from "@/components/session-row"
import { trpc } from "@/utils/trpc"

export default function WorkspaceDetail() {
  const { workspaceId } = useLocalSearchParams<{ workspaceId: string }>()
  const router = useRouter()
  const queryClient = useQueryClient()

  const { data, error, isPending } = useQuery(
    trpc.sessions.list.queryOptions({ workspaceId })
  )

  const invalidate = useCallback(() => {
    queryClient.invalidateQueries(
      trpc.sessions.list.queryFilter({ workspaceId })
    )
  }, [queryClient, workspaceId])

  const start = useMutation(
    trpc.sessions.start.mutationOptions({ onSuccess: invalidate })
  )
  const stop = useMutation(
    trpc.sessions.stop.mutationOptions({ onSuccess: invalidate })
  )
  const retry = useMutation(
    trpc.sessions.retry.mutationOptions({ onSuccess: invalidate })
  )

  const handleBack = useCallback(() => {
    router.back()
  }, [router])

  const handleStart = useCallback(
    (id: string) => {
      start.mutate({ id })
    },
    [start]
  )
  const handleStop = useCallback(
    (id: string) => {
      stop.mutate({ id })
    },
    [stop]
  )
  const handleRetry = useCallback(
    (id: string) => {
      retry.mutate({ id })
    },
    [retry]
  )

  return (
    <View className="flex-1 gap-4 bg-background px-6 pt-16">
      <View className="flex-row items-center justify-between">
        <Text className="font-geist-semibold text-2xl text-foreground">
          Sessions
        </Text>
        <Button onPress={handleBack} size="sm" variant="ghost">
          <Button.Label>Workspaces</Button.Label>
        </Button>
      </View>

      {isPending ? <Spinner /> : null}

      {error ? (
        <Text className="font-geist text-danger text-sm">{error.message}</Text>
      ) : null}

      {data && data.length === 0 ? (
        <View className="gap-1">
          <Text className="font-geist text-base text-foreground">
            No sessions yet.
          </Text>
          <Text className="font-geist text-muted text-sm">
            Start a task on desktop to run an agent session.
          </Text>
        </View>
      ) : null}

      {data?.map((session) => (
        <SessionRow
          key={session.id}
          onRetry={handleRetry}
          onStart={handleStart}
          onStop={handleStop}
          session={session}
        />
      ))}
    </View>
  )
}
