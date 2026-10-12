import { Button } from "@kloucy/ui/components/button"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { useCallback } from "react"
import { useTRPC } from "@/utils/trpc"

interface ReviewRowProps {
  review: {
    action: string
    artifactId: string | null
    id: string
    workspaceId: string
  }
}

export function ReviewRow({ review }: ReviewRowProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const invalidate = useCallback(() => {
    queryClient.invalidateQueries(
      trpc.reviews.list.queryFilter({ workspaceId: review.workspaceId })
    )
  }, [queryClient, review.workspaceId, trpc])

  const approve = useMutation(
    trpc.reviews.approve.mutationOptions({ onSuccess: invalidate })
  )
  const reject = useMutation(
    trpc.reviews.reject.mutationOptions({ onSuccess: invalidate })
  )

  const handleApprove = useCallback(() => {
    approve.mutate({ id: review.id })
  }, [approve, review.id])
  const handleReject = useCallback(() => {
    reject.mutate({ id: review.id })
  }, [reject, review.id])

  return (
    <div className="flex items-center justify-between gap-4 px-2 py-3">
      <div className="flex min-w-0 flex-col">
        <span className="truncate text-medium">
          {review.artifactId ?? "Changed files"}
        </span>
        <span className="text-caption text-muted-foreground">
          {review.action}
        </span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <Button onClick={handleApprove} size="sm" variant="secondary">
          Approve
        </Button>
        <Button onClick={handleReject} size="sm" variant="destructive">
          Reject
        </Button>
      </div>
    </div>
  )
}
