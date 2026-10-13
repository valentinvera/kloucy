import { Button, Input, Label, TextField } from "heroui-native"
import { useCallback, useState } from "react"
import { Text, View } from "react-native"

interface ReviewRowProps {
  onApprove: (id: string) => void
  onReject: (id: string, reason: string) => void
  review: {
    action: string
    artifactId: string | null
    id: string
    reason: string | null
  }
}

export function ReviewRow({ onApprove, onReject, review }: ReviewRowProps) {
  const [isRejecting, setIsRejecting] = useState(false)
  const [reason, setReason] = useState("")

  const handleApprove = useCallback(
    () => onApprove(review.id),
    [onApprove, review.id]
  )
  const handleStartReject = useCallback(() => {
    setReason(review.reason ?? "")
    setIsRejecting(true)
  }, [review.reason])
  const handleCancelReject = useCallback(() => {
    setIsRejecting(false)
    setReason("")
  }, [])
  const handleConfirmReject = useCallback(() => {
    const trimmed = reason.trim()

    if (!trimmed) {
      return
    }

    onReject(review.id, trimmed)
    setIsRejecting(false)
    setReason("")
  }, [onReject, reason, review.id])

  return (
    <View className="gap-2 py-3">
      <Text className="font-geist text-base text-foreground">
        {review.artifactId ?? "Changed files"}
      </Text>
      <Text className="font-geist text-muted text-xs">{review.action}</Text>
      {review.reason ? (
        <Text className="font-geist text-muted text-xs">{review.reason}</Text>
      ) : null}

      {isRejecting ? (
        <View className="gap-2">
          <TextField>
            <Label>Why reject this change?</Label>
            <Input
              onChangeText={setReason}
              placeholder="What is wrong with this change?"
              value={reason}
            />
          </TextField>
          <View className="flex-row gap-2">
            <Button
              isDisabled={reason.trim().length === 0}
              onPress={handleConfirmReject}
              size="sm"
              variant="danger-soft"
            >
              <Button.Label>Reject change</Button.Label>
            </Button>
            <Button onPress={handleCancelReject} size="sm" variant="ghost">
              <Button.Label>Cancel</Button.Label>
            </Button>
          </View>
        </View>
      ) : (
        <View className="flex-row gap-2">
          <Button onPress={handleApprove} size="sm" variant="secondary">
            <Button.Label>Approve</Button.Label>
          </Button>
          <Button onPress={handleStartReject} size="sm" variant="danger-soft">
            <Button.Label>Reject</Button.Label>
          </Button>
        </View>
      )}
    </View>
  )
}
