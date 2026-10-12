import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { useQuery } from "@tanstack/react-query"
import { ReviewRow } from "@/components/review-row"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceReviewsProps {
  workspaceId: string
}

export function WorkspaceReviews({ workspaceId }: WorkspaceReviewsProps) {
  const trpc = useTRPC()
  const { data, error, isPending } = useQuery(
    trpc.reviews.list.queryOptions({ workspaceId })
  )

  return (
    <div className="mt-10">
      <h2 className="font-medium text-medium">Reviews</h2>
      <div className="mt-4">
        {isPending ? (
          <p className="text-medium text-muted-foreground">Loading reviews…</p>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        ) : null}

        {data && data.length === 0 ? (
          <div className="border-border/60 border-t py-6">
            <p className="text-medium">Nothing to review yet.</p>
            <p className="mt-1 text-caption text-muted-foreground">
              Completed sessions with changes appear here.
            </p>
          </div>
        ) : null}

        {data && data.length > 0 ? (
          <div className="divide-y divide-border/60 border-border/60 border-t">
            {data.map((review) => (
              <ReviewRow key={review.id} review={review} />
            ))}
          </div>
        ) : null}
      </div>
    </div>
  )
}
