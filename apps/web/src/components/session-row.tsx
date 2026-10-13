import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback, useState } from "react"
import { nextLogs, type SessionOutput } from "@/lib/session-output"
import { useTRPC, useTRPCClient } from "@/utils/trpc"

const VISIBLE_LOGS = 8
const OUTPUT_REFETCH_MS = 3000

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
  const trpcClient = useTRPCClient()
  const queryClient = useQueryClient()
  const [isExpanded, setIsExpanded] = useState(false)

  const outputKey = ["session-output", session.id]

  // Only the lines that are new since the last poll travel over the wire, and
  // the response carries the session state, so the row stays live while its
  // output is open without reloading the whole list.
  const output = useQuery({
    enabled: isExpanded,
    queryFn: async () => {
      const previous = queryClient.getQueryData<SessionOutput>(outputKey)
      const held = previous?.logs ?? []

      const stream = await trpcClient.sessions.stream.query({
        from: held.length,
        sessionId: session.id,
      })

      return {
        logs: nextLogs(
          held,
          stream.logs.map((log) => ({
            id: log.id,
            message: log.message,
            stream: log.stream,
          }))
        ),
        status: stream.session.status,
      }
    },
    queryKey: outputKey,
    refetchInterval: OUTPUT_REFETCH_MS,
  })

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
  const handleToggleOutput = useCallback(() => {
    setIsExpanded((value) => !value)
  }, [])

  const logs = output.data?.logs ?? []
  const visibleLogs = logs.slice(-VISIBLE_LOGS)
  const status = output.data?.status ?? session.status

  return (
    <div className="flex flex-col gap-2 px-2 py-3">
      <div className="flex items-center justify-between gap-4">
        <div className="flex min-w-0 flex-col">
          <span className="truncate text-medium">
            {session.prompt ?? "Untitled session"}
          </span>
          <span className="text-caption text-muted-foreground">{status}</span>
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
          <Button onClick={handleToggleOutput} size="sm" variant="ghost">
            {isExpanded ? "Hide output" : "Output"}
          </Button>
        </div>
      </div>

      {isExpanded ? (
        <div className="flex flex-col gap-1 border border-border/60 bg-card p-3 font-mono text-caption">
          {output.isPending ? (
            <p className="text-muted-foreground">Loading output…</p>
          ) : null}

          {output.error ? (
            <p className="text-destructive">{output.error.message}</p>
          ) : null}

          {output.data && logs.length === 0 ? (
            <p className="text-muted-foreground">No output yet.</p>
          ) : null}

          {visibleLogs.map((line) => (
            <p
              className={
                line.stream === "stderr"
                  ? "text-destructive"
                  : "text-muted-foreground"
              }
              key={line.id}
            >
              {line.message}
            </p>
          ))}
        </div>
      ) : null}
    </div>
  )
}
