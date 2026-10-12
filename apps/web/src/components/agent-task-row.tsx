import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { useCallback } from "react"
import { useTRPC } from "@/utils/trpc"

interface AgentTaskRowProps {
  task: {
    id: string
    source: string
    status: string
    title: string
  }
  workspaceId: string
}

export function AgentTaskRow({ task, workspaceId }: AgentTaskRowProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const startSession = useMutation(
    trpc.sessions.create.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.sessions.list.queryFilter({ workspaceId })
        )
      },
    })
  )

  const handleStart = useCallback(() => {
    startSession.mutate({ taskId: task.id, workspaceId })
  }, [startSession, task.id, workspaceId])

  return (
    <li>
      <div className="flex items-center justify-between gap-4 px-2 py-3">
        <span className="flex min-w-0 flex-col">
          <span className="truncate text-medium">{task.title}</span>
          <span className="text-caption text-muted-foreground">
            {task.source}
          </span>
        </span>
        <span className="flex shrink-0 items-center gap-3">
          <span className="text-caption text-muted-foreground">
            {task.status}
          </span>
          <Button
            disabled={startSession.isPending}
            onClick={handleStart}
            size="sm"
            variant="secondary"
          >
            Start session
          </Button>
        </span>
      </div>
    </li>
  )
}
