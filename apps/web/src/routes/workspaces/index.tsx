import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Badge } from "@kloucy/ui/components/badge"
import { useQuery } from "@tanstack/react-query"
import { createFileRoute, Link } from "@tanstack/react-router"
import { WorkspaceCreateDialog } from "@/components/workspace-create-dialog"
import { formatDate, stageClass, stageLabel } from "@/lib/workspace-display"
import { useTRPC } from "@/utils/trpc"

export const Route = createFileRoute("/workspaces/")({
  component: WorkspacesIndexComponent,
})

function WorkspacesIndexComponent() {
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(
    trpc.workspaces.list.queryOptions()
  )

  return (
    <section className="mx-auto w-full max-w-3xl px-6 py-10">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="font-medium text-section">Workspaces</h1>
          <p className="mt-1 text-medium text-muted-foreground">
            Local projects Kloucy can supervise.
          </p>
        </div>
        <WorkspaceCreateDialog />
      </div>

      <div className="mt-8">
        {isPending ? (
          <p className="text-medium text-muted-foreground">
            Loading workspaces…
          </p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {data && data.length === 0 ? (
          <div className="border-border/60 border-t py-10">
            <p className="text-medium">Open your first workspace.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Create a workspace to run and supervise local agents.
            </p>
          </div>
        ) : null}

        {data && data.length > 0 ? (
          <ul className="divide-y divide-border/60 border-border/60 border-t">
            {data.map((workspace) => (
              <li key={workspace.id}>
                <Link
                  className="flex items-center justify-between gap-4 px-2 py-3 transition-colors hover:bg-muted/40"
                  params={{ workspaceId: workspace.id }}
                  to="/workspaces/$workspaceId"
                >
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate text-medium">
                      {workspace.name}
                    </span>
                    <span className="truncate font-mono text-caption text-muted-foreground">
                      {workspace.localPath}
                    </span>
                  </span>
                  <span className="flex shrink-0 items-center gap-3">
                    <Badge
                      className={stageClass(workspace.stage)}
                      variant="outline"
                    >
                      {stageLabel(workspace.stage)}
                    </Badge>
                    <span className="text-caption text-muted-foreground">
                      {formatDate(workspace.updatedAt)}
                    </span>
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  )
}
