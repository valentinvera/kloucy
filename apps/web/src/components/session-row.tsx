import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { useCallback } from "react"
import { useTRPC } from "@/utils/trpc"

interface SessionRowProps {
  session: {
    id: string
    prompt: string | null
    status: string
    workspaceId: string
  }
}

export function SessionRow({ session }: SessionRowProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const invalidate = useCallback(() => {
    queryClient.invalidateQueries(
      trpc.sessions.list.queryFilter({ workspaceId: session.workspaceId })
    )
  }, [queryClient, session.workspaceId, trpc])

  const start = useMutation(
    trpc.sessions.start.mutationOptions({ onSuccess: invalidate })
  )
  const stop = useMutation(
    trpc.sessions.stop.mutationOptions({ onSuccess: invalidate })
  )
  const retry = useMutation(
    trpc.sessions.retry.mutationOptions({ onSuccess: invalidate })
  )

  const handleStart = useCallback(() => {
    start.mutate({ id: session.id })
  }, [session.id, start])
  const handleStop = useCallback(() => {
    stop.mutate({ id: session.id })
  }, [session.id, stop])
  const handleRetry = useCallback(() => {
    retry.mutate({ id: session.id })
  }, [session.id, retry])

  return (
    <div className="flex items-center justify-between gap-4 px-2 py-3">
      <div className="flex min-w-0 flex-col">
        <span className="truncate text-medium">
          {session.prompt ?? "Untitled session"}
        </span>
        <span className="text-caption text-muted-foreground">
          {session.status}
        </span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <Button onClick={handleStart} size="sm" variant="secondary">
          Start
        </Button>
        <Button onClick={handleStop} size="sm" variant="secondary">
          Stop
        </Button>
        <Button onClick={handleRetry} size="sm" variant="ghost">
          Retry
        </Button>
      </div>
    </div>
  )
}
