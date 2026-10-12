import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { useQuery } from "@tanstack/react-query"
import { AgentTaskCreateDialog } from "@/components/agent-task-create-dialog"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceInboxProps {
  workspaceId: string
}

export function WorkspaceInbox({ workspaceId }: WorkspaceInboxProps) {
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(
    trpc.agentTasks.list.queryOptions({ workspaceId })
  )

  return (
    <div className="mt-10">
      <div className="flex items-center justify-between gap-4">
        <h2 className="font-medium text-medium">Inbox</h2>
        <AgentTaskCreateDialog workspaceId={workspaceId} />
      </div>
      <div className="mt-4">
        {isPending ? (
          <p className="text-medium text-muted-foreground">Loading tasks…</p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {data && data.length === 0 ? (
          <div className="border-border/60 border-t py-6">
            <p className="text-medium">No captured tasks yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Capture a task to assign it to an agent.
            </p>
          </div>
        ) : null}

        {data && data.length > 0 ? (
          <ul className="divide-y divide-border/60 border-border/60 border-t">
            {data.map((task) => (
              <li key={task.id}>
                <div className="flex items-center justify-between gap-4 px-2 py-3">
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate text-medium">{task.title}</span>
                    <span className="text-caption text-muted-foreground">
                      {task.source}
                    </span>
                  </span>
                  <span className="shrink-0 text-caption text-muted-foreground">
                    {task.status}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </div>
  )
}
