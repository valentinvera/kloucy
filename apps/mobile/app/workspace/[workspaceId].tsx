import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useLocalSearchParams, useRouter } from "expo-router"
import { Button, Spinner } from "heroui-native"
import { useCallback, useMemo } from "react"
import { ScrollView, Text, View } from "react-native"
import { NextActions } from "@/components/next-actions"
import { QuickInstruction } from "@/components/quick-instruction"
import { ReviewRow } from "@/components/review-row"
import { SafetyModeSwitch } from "@/components/safety-mode-switch"
import { SessionRow } from "@/components/session-row"
import { WorkspaceSummary } from "@/components/workspace-summary"
import { nextActions } from "@/lib/next-actions"
import { sessionCounts } from "@/lib/session-counts"
import { trpc } from "@/utils/trpc"

export default function WorkspaceDetail() {
  const { workspaceId } = useLocalSearchParams<{ workspaceId: string }>()
  const router = useRouter()
  const queryClient = useQueryClient()

  const workspace = useQuery(
    trpc.workspaces.get.queryOptions({ id: workspaceId })
  )
  const sessions = useQuery(trpc.sessions.list.queryOptions({ workspaceId }))
  const reviews = useQuery(trpc.reviews.list.queryOptions({ workspaceId }))

  const counts = useMemo(
    () => sessionCounts(sessions.data ?? []),
    [sessions.data]
  )

  const actions = useMemo(
    () =>
      nextActions({
        reviews: reviews.data ?? [],
        sessions: sessions.data ?? [],
      }),
    [reviews.data, sessions.data]
  )

  const invalidateSessions = useCallback(() => {
    queryClient.invalidateQueries(
      trpc.sessions.list.queryFilter({ workspaceId })
    )
  }, [queryClient, workspaceId])

  const invalidateReviews = useCallback(() => {
    queryClient.invalidateQueries(
      trpc.reviews.list.queryFilter({ workspaceId })
    )
  }, [queryClient, workspaceId])

  const start = useMutation(
    trpc.sessions.start.mutationOptions({ onSuccess: invalidateSessions })
  )
  const stop = useMutation(
    trpc.sessions.stop.mutationOptions({ onSuccess: invalidateSessions })
  )
  const retry = useMutation(
    trpc.sessions.retry.mutationOptions({ onSuccess: invalidateSessions })
  )
  const sendInstruction = useMutation(
    trpc.sessions.sendInstruction.mutationOptions({
      onSuccess: invalidateSessions,
    })
  )
  const updateSafetyMode = useMutation(
    trpc.workspaces.update.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.workspaces.get.queryFilter({ id: workspaceId })
        )
      },
    })
  )
  const approve = useMutation(
    trpc.reviews.approve.mutationOptions({ onSuccess: invalidateReviews })
  )
  const reject = useMutation(
    trpc.reviews.reject.mutationOptions({ onSuccess: invalidateReviews })
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
  const handleSendInstruction = useCallback(
    (id: string, instruction: string) => {
      sendInstruction.mutate({ id, instruction })
    },
    [sendInstruction]
  )
  const handleGuardCommandsChange = useCallback(
    (guarded: boolean) => {
      updateSafetyMode.mutate({
        id: workspaceId,
        safetyMode: guarded ? "guarded" : "open",
      })
    },
    [updateSafetyMode, workspaceId]
  )
  const handleApprove = useCallback(
    (id: string) => {
      approve.mutate({ id })
    },
    [approve]
  )
  const handleReject = useCallback(
    (id: string, reason: string) => {
      reject.mutate({ id, reason })
    },
    [reject]
  )

  return (
    <ScrollView className="flex-1 bg-background">
      <View className="gap-4 px-6 pt-16 pb-16">
        <View className="flex-row items-start justify-between gap-4">
          <WorkspaceSummary
            counts={counts}
            localPath={workspace.data?.localPath}
            name={workspace.data?.name ?? "Workspace"}
            safetyMode={workspace.data?.safetyMode}
          />
          <Button onPress={handleBack} size="sm" variant="ghost">
            <Button.Label>Workspaces</Button.Label>
          </Button>
        </View>

        <SafetyModeSwitch
          isGuarded={workspace.data?.safetyMode === "guarded"}
          onChange={handleGuardCommandsChange}
        />

        <NextActions actions={actions} />

        <Text className="font-geist-semibold text-foreground text-lg">
          Sessions
        </Text>

        {sessions.isPending ? <Spinner /> : null}

        {sessions.error ? (
          <Text className="font-geist text-danger text-sm">
            {sessions.error.message}
          </Text>
        ) : null}

        {sessions.data && sessions.data.length === 0 ? (
          <Text className="font-geist text-muted text-sm">
            No sessions yet. Start a task on desktop.
          </Text>
        ) : null}

        {sessions.data?.map((session) => (
          <SessionRow
            key={session.id}
            onRetry={handleRetry}
            onStart={handleStart}
            onStop={handleStop}
            session={session}
          />
        ))}

        <QuickInstruction
          isSending={sendInstruction.isPending}
          onSend={handleSendInstruction}
          sessions={sessions.data ?? []}
        />

        <Text className="mt-4 font-geist-semibold text-foreground text-lg">
          Reviews
        </Text>

        {reviews.isPending ? <Spinner /> : null}

        {reviews.error ? (
          <Text className="font-geist text-danger text-sm">
            {reviews.error.message}
          </Text>
        ) : null}

        {reviews.data && reviews.data.length === 0 ? (
          <Text className="font-geist text-muted text-sm">
            Nothing to review yet.
          </Text>
        ) : null}

        {reviews.data?.map((review) => (
          <ReviewRow
            key={review.id}
            onApprove={handleApprove}
            onReject={handleReject}
            review={review}
          />
        ))}
      </View>
    </ScrollView>
  )
}
