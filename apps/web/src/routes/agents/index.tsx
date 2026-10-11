import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { useQuery } from "@tanstack/react-query"
import { createFileRoute } from "@tanstack/react-router"
import { AgentCreateDialog } from "@/components/agent-create-dialog"
import { useTRPC } from "@/utils/trpc"

export const Route = createFileRoute("/agents/")({
  component: AgentsIndexComponent,
})

function AgentsIndexComponent() {
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(trpc.agents.list.queryOptions())

  return (
    <section className="mx-auto w-full max-w-3xl px-6 py-10">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="font-medium text-section">Agents</h1>
          <p className="mt-1 text-medium text-muted-foreground">
            Local agents Kloucy can run tasks with.
          </p>
        </div>
        <AgentCreateDialog />
      </div>

      <div className="mt-8">
        {isPending ? (
          <p className="text-medium text-muted-foreground">Loading agents…</p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {data && data.length === 0 ? (
          <div className="border-border/60 border-t py-10">
            <p className="text-medium">No agents configured yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Register an agent to run tasks in a workspace.
            </p>
          </div>
        ) : null}

        {data && data.length > 0 ? (
          <ul className="divide-y divide-border/60 border-border/60 border-t">
            {data.map((agent) => (
              <li key={agent.id}>
                <div className="flex items-center justify-between gap-4 px-2 py-3">
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate text-medium">{agent.name}</span>
                    <span className="truncate font-mono text-caption text-muted-foreground">
                      {agent.provider}
                    </span>
                  </span>
                  <span className="shrink-0 text-caption text-muted-foreground">
                    {agent.status}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  )
}
