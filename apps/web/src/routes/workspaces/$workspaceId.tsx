import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Badge } from "@kloucy/ui/components/badge"
import { useQuery } from "@tanstack/react-query"
import { createFileRoute, Link } from "@tanstack/react-router"
import { formatDate, stageClass, stageLabel } from "@/lib/workspace-display"
import { useTRPC } from "@/utils/trpc"

export const Route = createFileRoute("/workspaces/$workspaceId")({
  component: WorkspaceDetailComponent,
})

function WorkspaceDetailComponent() {
  const { workspaceId } = Route.useParams()
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(
    trpc.workspaces.get.queryOptions({ id: workspaceId })
  )

  return (
    <section className="mx-auto w-full max-w-3xl px-6 py-10">
      <Link
        className="text-caption text-muted-foreground hover:text-foreground"
        to="/workspaces"
      >
        ← Workspaces
      </Link>

      {isPending ? (
        <p className="mt-6 text-medium text-muted-foreground">
          Opening workspace…
        </p>
      ) : null}

      {error ? (
        <Alert className="mt-6" variant="destructive">
          <AlertDescription>{error.message}</AlertDescription>
        </Alert>
      ) : null}

      {data ? (
        <div className="mt-6">
          <div className="flex items-center gap-3">
            <h1 className="font-medium text-section">{data.name}</h1>
            <Badge className={stageClass(data.stage)} variant="outline">
              {stageLabel(data.stage)}
            </Badge>
          </div>
          <p className="mt-2 font-mono text-caption text-muted-foreground">
            {data.localPath}
          </p>
          {data.description ? (
            <p className="mt-4 text-medium">{data.description}</p>
          ) : null}

          <dl className="mt-8 grid grid-cols-1 gap-x-8 gap-y-4 border-border/60 border-t pt-6 sm:grid-cols-2">
            <div>
              <dt className="text-caption text-muted-foreground">
                Project type
              </dt>
              <dd className="mt-1 text-medium">
                {data.projectType ?? "Not set"}
              </dd>
            </div>
            <div>
              <dt className="text-caption text-muted-foreground">
                Package manager
              </dt>
              <dd className="mt-1 text-medium">
                {data.packageManager ?? "Not set"}
              </dd>
            </div>
            <div>
              <dt className="text-caption text-muted-foreground">Created</dt>
              <dd className="mt-1 text-medium">{formatDate(data.createdAt)}</dd>
            </div>
            <div>
              <dt className="text-caption text-muted-foreground">
                Last activity
              </dt>
              <dd className="mt-1 text-medium">{formatDate(data.updatedAt)}</dd>
            </div>
          </dl>

          <p className="mt-8 text-caption text-muted-foreground">
            Agent sessions, reviews, and controls for this workspace arrive
            next.
          </p>
        </div>
      ) : null}
    </section>
  )
}
