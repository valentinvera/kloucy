import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Badge } from "@kloucy/ui/components/badge"
import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback } from "react"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceReadinessProps {
  workspaceId: string
}

const STATUS_LABEL: Record<string, string> = {
  almost_ready: "Almost ready",
  not_ready: "Not ready",
  ready_to_run: "Ready to run",
}

const STATUS_CLASS: Record<string, string> = {
  almost_ready: "border-transparent bg-warning text-warning-foreground",
  not_ready: "border-transparent bg-secondary text-secondary-foreground",
  ready_to_run: "border-transparent bg-success text-success-foreground",
}

export function WorkspaceReadiness({ workspaceId }: WorkspaceReadinessProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const { data, error, isPending } = useQuery(
    trpc.readiness.latest.queryOptions({ workspaceId })
  )

  const calculate = useMutation(
    trpc.readiness.calculate.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.readiness.latest.queryFilter({ workspaceId })
        )
      },
    })
  )

  const handleCalculate = useCallback(() => {
    calculate.mutate({ workspaceId })
  }, [calculate, workspaceId])

  return (
    <div className="mt-10">
      <div className="flex items-center justify-between gap-4">
        <h2 className="font-medium text-medium">Readiness</h2>
        <Button
          disabled={calculate.isPending}
          onClick={handleCalculate}
          size="sm"
          variant="secondary"
        >
          {calculate.isPending ? "Calculating…" : "Calculate"}
        </Button>
      </div>

      <div className="mt-4">
        {isPending ? (
          <p className="text-medium text-muted-foreground">
            Loading readiness…
          </p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {isPending || error || data ? null : (
          <div className="border-border/60 border-t py-6">
            <p className="text-medium">No readiness check yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Calculate readiness to see blockers before running agents.
            </p>
          </div>
        )}

        {data ? (
          <div className="border-border/60 border-t pt-6">
            <div className="flex items-baseline gap-3">
              <span className="font-medium text-3xl">{data.score}</span>
              <Badge
                className={STATUS_CLASS[data.status] ?? ""}
                variant="outline"
              >
                {STATUS_LABEL[data.status] ?? data.status}
              </Badge>
            </div>

            {data.blockers && data.blockers.length > 0 ? (
              <div className="mt-4">
                <p className="text-caption text-muted-foreground">Blockers</p>
                <ul className="mt-1 flex flex-col gap-1">
                  {data.blockers.map((blocker) => (
                    <li className="text-medium" key={blocker}>
                      {blocker}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}

            {data.notRequiredYet && data.notRequiredYet.length > 0 ? (
              <div className="mt-4">
                <p className="text-caption text-muted-foreground">
                  Not required yet
                </p>
                <ul className="mt-1 flex flex-col gap-1">
                  {data.notRequiredYet.map((item) => (
                    <li
                      className="text-medium text-muted-foreground"
                      key={item}
                    >
                      {item}
                    </li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  )
}
