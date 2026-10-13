import { Button } from "@kloucy/ui/components/button"
import { Field, FieldLabel } from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { type ChangeEvent, useCallback, useState } from "react"
import { useTRPC } from "@/utils/trpc"

interface ReviewRowProps {
  review: {
    action: string
    artifactId: string | null
    id: string
    reason: string | null
    workspaceId: string
  }
}

export function ReviewRow({ review }: ReviewRowProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const [isRejecting, setIsRejecting] = useState(false)
  const [reason, setReason] = useState("")

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
  const handleStartReject = useCallback(() => {
    setReason(review.reason ?? "")
    setIsRejecting(true)
  }, [review.reason])
  const handleCancelReject = useCallback(() => {
    setIsRejecting(false)
    setReason("")
  }, [])
  const handleReasonChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setReason(event.target.value)
    },
    []
  )
  const handleConfirmReject = useCallback(() => {
    const trimmed = reason.trim()

    if (!trimmed) {
      return
    }

    reject.mutate({ id: review.id, reason: trimmed })
    setIsRejecting(false)
    setReason("")
  }, [reason, reject, review.id])

  return (
    <div className="flex flex-col gap-2 px-2 py-3">
      <div className="flex items-center justify-between gap-4">
        <div className="flex min-w-0 flex-col">
          <span className="truncate text-medium">
            {review.artifactId ?? "Changed files"}
          </span>
          <span className="text-caption text-muted-foreground">
            {review.action}
          </span>
          {review.reason ? (
            <span className="text-caption text-muted-foreground">
              {review.reason}
            </span>
          ) : null}
        </div>
        {isRejecting ? null : (
          <div className="flex shrink-0 items-center gap-2">
            <Button
              disabled={approve.isPending}
              onClick={handleApprove}
              size="sm"
              variant="secondary"
            >
              Approve
            </Button>
            <Button
              disabled={reject.isPending}
              onClick={handleStartReject}
              size="sm"
              variant="destructive"
            >
              Reject
            </Button>
          </div>
        )}
      </div>

      {isRejecting ? (
        <div className="flex flex-col gap-2 sm:flex-row sm:items-end">
          <Field className="flex-1">
            <FieldLabel htmlFor={`reject-reason-${review.id}`}>
              Why reject this change?
            </FieldLabel>
            <Input
              id={`reject-reason-${review.id}`}
              onChange={handleReasonChange}
              placeholder="What is wrong with this change?"
              value={reason}
            />
          </Field>
          <div className="flex shrink-0 items-center gap-2">
            <Button
              disabled={reason.trim().length === 0 || reject.isPending}
              onClick={handleConfirmReject}
              size="sm"
              variant="destructive"
            >
              Reject change
            </Button>
            <Button
              disabled={reject.isPending}
              onClick={handleCancelReject}
              size="sm"
              variant="ghost"
            >
              Cancel
            </Button>
          </div>
        </div>
      ) : null}
    </div>
  )
}
