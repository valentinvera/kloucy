import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useCallback } from "react"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceWeeklyReviewProps {
  workspaceId: string
}

function ReviewItems({
  items,
  title,
}: {
  items: string[] | null
  title: string
}) {
  if (!items || items.length === 0) {
    return null
  }

  return (
    <div className="mt-4">
      <p className="text-caption text-muted-foreground">{title}</p>
      <ul className="mt-1 flex flex-col gap-1">
        {items.map((item) => (
          <li className="text-medium" key={item}>
            {item}
          </li>
        ))}
      </ul>
    </div>
  )
}

export function WorkspaceWeeklyReview({
  workspaceId,
}: WorkspaceWeeklyReviewProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const sessions = useQuery(trpc.sessions.list.queryOptions({ workspaceId }))
  const reviews = useQuery(trpc.reviews.list.queryOptions({ workspaceId }))
  const latest = useQuery(
    trpc.weeklyReviews.latest.queryOptions({ workspaceId })
  )

  const createReview = useMutation(
    trpc.weeklyReviews.create.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.weeklyReviews.latest.queryFilter({ workspaceId })
        )
      },
    })
  )

  const handleGenerate = useCallback(() => {
    const sessionList = sessions.data ?? []
    const reviewList = reviews.data ?? []

    const done = sessionList.filter((session) => session.status === "done")
    const running = sessionList.filter(
      (session) => session.status === "running"
    )
    const waiting = sessionList.filter(
      (session) => session.status === "waiting"
    )

    createReview.mutate({
      blockers: waiting.map((session) => session.prompt ?? "Untitled session"),
      nextActions: ["Review waiting sessions", "Start a task from the inbox"],
      rejectedChanges: reviewList
        .filter((entry) => entry.action === "rejected")
        .map((entry) => entry.artifactId ?? "Changed files"),
      shipped: done.map((session) => session.prompt ?? "Untitled session"),
      summary: `${done.length} done, ${running.length} running, ${waiting.length} waiting`,
      workspaceId,
    })
  }, [createReview, reviews.data, sessions.data, workspaceId])

  const review = latest.data

  return (
    <div className="mt-10">
      <div className="flex items-center justify-between gap-4">
        <h2 className="font-medium text-medium">Weekly review</h2>
        <Button
          disabled={createReview.isPending}
          onClick={handleGenerate}
          size="sm"
          variant="secondary"
        >
          {createReview.isPending ? "Generating…" : "Generate review"}
        </Button>
      </div>

      <div className="mt-4">
        {latest.isPending ? (
          <p className="text-medium text-muted-foreground">
            Loading weekly review…
          </p>
        ) : null}

        {latest.isPending || review ? null : (
          <div className="border-border/60 border-t py-6">
            <p className="text-medium">No weekly review yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Generate a review from this week's sessions and reviews.
            </p>
          </div>
        )}

        {review ? (
          <div className="border-border/60 border-t pt-6">
            {review.summary ? (
              <p className="text-medium">{review.summary}</p>
            ) : null}
            <ReviewItems items={review.shipped} title="Shipped" />
            <ReviewItems items={review.blockers} title="Blockers" />
            <ReviewItems
              items={review.rejectedChanges}
              title="Rejected changes"
            />
            <ReviewItems items={review.nextActions} title="Next actions" />
          </div>
        ) : null}
      </div>
    </div>
  )
}
