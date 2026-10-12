import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { useQuery } from "@tanstack/react-query"
import { SessionRow } from "@/components/session-row"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceSessionsProps {
  workspaceId: string
}

export function WorkspaceSessions({ workspaceId }: WorkspaceSessionsProps) {
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(
    trpc.sessions.list.queryOptions({ workspaceId })
  )

  return (
    <div className="mt-10">
      <h2 className="font-medium text-medium">Sessions</h2>
      <div className="mt-4">
        {isPending ? (
          <p className="text-medium text-muted-foreground">Loading sessions…</p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {data && data.length === 0 ? (
          <div className="border-border/60 border-t py-6">
            <p className="text-medium">No sessions yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Start a task to run an agent session.
            </p>
          </div>
        ) : null}

        {data && data.length > 0 ? (
          <div className="divide-y divide-border/60 border-border/60 border-t">
            {data.map((session) => (
              <SessionRow key={session.id} session={session} />
            ))}
          </div>
        ) : null}
      </div>
    </div>
  )
}
