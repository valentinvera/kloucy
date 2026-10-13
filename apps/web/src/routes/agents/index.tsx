import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { createFileRoute } from "@tanstack/react-router"
import { useCallback, useState } from "react"
import { AgentCreateDialog } from "@/components/agent-create-dialog"
import { useTRPC } from "@/utils/trpc"

export const Route = createFileRoute("/agents/")({
  component: AgentsIndexComponent,
})

interface DetectedAgent {
  executablePath: string
  name: string
  provider: string
}

interface TestResult {
  ok: boolean
  output: string
}

function DetectedList({ detected }: { detected: DetectedAgent[] }) {
  if (detected.length === 0) {
    return (
      <p className="text-caption text-muted-foreground">
        No known agent CLI was found on the machine running the API.
      </p>
    )
  }

  return (
    <ul className="flex flex-col gap-1">
      {detected.map((entry) => (
        <li className="flex items-baseline gap-2" key={entry.provider}>
          <span className="text-medium">{entry.name}</span>
          <span className="text-caption text-muted-foreground">
            {entry.provider}
          </span>
          <span className="truncate font-mono text-caption text-muted-foreground">
            {entry.executablePath}
          </span>
        </li>
      ))}
    </ul>
  )
}

interface AgentRowProps {
  agent: {
    id: string
    name: string
    provider: string
    status: string
  }
  isTesting: boolean
  onTest: (id: string) => void
  result: TestResult | undefined
}

function AgentRow({ agent, isTesting, onTest, result }: AgentRowProps) {
  const handleTest = useCallback(() => {
    onTest(agent.id)
  }, [agent.id, onTest])

  return (
    <div className="flex items-center justify-between gap-4 px-2 py-3">
      <span className="flex min-w-0 flex-col">
        <span className="truncate text-medium">{agent.name}</span>
        <span className="truncate font-mono text-caption text-muted-foreground">
          {agent.provider}
        </span>
        {result ? (
          <span
            className={
              result.ok
                ? "truncate font-mono text-caption text-success-foreground"
                : "truncate font-mono text-caption text-destructive"
            }
          >
            {result.output}
          </span>
        ) : null}
      </span>
      <span className="flex shrink-0 items-center gap-2">
        <span className="text-caption text-muted-foreground">
          {agent.status}
        </span>
        <Button
          disabled={isTesting}
          onClick={handleTest}
          size="sm"
          variant="ghost"
        >
          Test
        </Button>
      </span>
    </div>
  )
}

function AgentsIndexComponent() {
  const trpc = useTRPC()
  const queryClient = useQueryClient()
  const { data, error, isPending } = useQuery(trpc.agents.list.queryOptions())

  const [detected, setDetected] = useState<DetectedAgent[] | null>(null)
  const [results, setResults] = useState<Record<string, TestResult>>({})

  const invalidateAgents = useCallback(() => {
    queryClient.invalidateQueries(trpc.agents.list.queryFilter())
  }, [queryClient, trpc])

  const testCommand = useMutation(
    trpc.agents.testCommand.mutationOptions({
      onSuccess: async (result, variables) => {
        setResults((current) => ({ ...current, [variables.id]: result }))
        await invalidateAgents()
      },
    })
  )

  const handleTest = useCallback(
    (id: string) => {
      testCommand.mutate({ id })
    },
    [testCommand]
  )

  const handleDetect = useCallback(async () => {
    const found = await queryClient.fetchQuery(
      trpc.agents.detect.queryOptions()
    )

    setDetected(found)
  }, [queryClient, trpc])

  return (
    <section className="mx-auto w-full max-w-3xl px-6 py-10">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="font-medium text-section">Agents</h1>
          <p className="mt-1 text-medium text-muted-foreground">
            Local agents Kloucy can run tasks with.
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <Button onClick={handleDetect} size="sm" variant="secondary">
            Detect agents
          </Button>
          <AgentCreateDialog />
        </div>
      </div>

      {detected ? (
        <div className="mt-6 border-border/60 border-t pt-4">
          <p className="text-caption text-muted-foreground">
            Found on the machine running the API. Register the ones you want to
            use.
          </p>
          <div className="mt-2">
            <DetectedList detected={detected} />
          </div>
        </div>
      ) : null}

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
                <AgentRow
                  agent={agent}
                  isTesting={testCommand.isPending}
                  onTest={handleTest}
                  result={results[agent.id]}
                />
              </li>
            ))}
          </ul>
        ) : null}
      </div>
    </section>
  )
}
