import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { useQuery } from "@tanstack/react-query"
import { DecisionCreateDialog } from "@/components/decision-create-dialog"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceDecisionsProps {
  workspaceId: string
}

export function WorkspaceDecisions({ workspaceId }: WorkspaceDecisionsProps) {
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(
    trpc.decisions.list.queryOptions({ workspaceId })
  )

  return (
    <div className="mt-10">
      <div className="flex items-center justify-between gap-4">
        <h2 className="font-medium text-medium">Decision log</h2>
        <DecisionCreateDialog workspaceId={workspaceId} />
      </div>
      <div className="mt-4">
        {isPending ? (
          <p className="text-medium text-muted-foreground">
            Loading decisions…
          </p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {data && data.length === 0 ? (
          <div className="border-border/60 border-t py-6">
            <p className="text-medium">No decisions yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Record scope, permission, run, or review decisions here.
            </p>
          </div>
        ) : null}

        {data && data.length > 0 ? (
          <ul className="divide-y divide-border/60 border-border/60 border-t">
            {data.map((decision) => (
              <li key={decision.id}>
                <div className="flex items-center justify-between gap-4 px-2 py-3">
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate text-medium">
                      {decision.title}
                    </span>
                    {decision.summary ? (
                      <span className="truncate text-caption text-muted-foreground">
                        {decision.summary}
                      </span>
                    ) : null}
                  </span>
                  <span className="shrink-0 text-caption text-muted-foreground">
                    {decision.type}
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
