import { Button } from "heroui-native"
import { useCallback } from "react"
import { Text, View } from "react-native"

interface ReviewRowProps {
  onApprove: (id: string) => void
  onReject: (id: string) => void
  review: {
    action: string
    artifactId: string | null
    id: string
  }
}

export function ReviewRow({ onApprove, onReject, review }: ReviewRowProps) {
  const handleApprove = useCallback(
    () => onApprove(review.id),
    [onApprove, review.id]
  )
  const handleReject = useCallback(
    () => onReject(review.id),
    [onReject, review.id]
  )

  return (
    <View className="gap-2 py-3">
      <Text className="font-geist text-base text-foreground">
        {review.artifactId ?? "Changed files"}
      </Text>
      <Text className="font-geist text-muted text-xs">{review.action}</Text>
      <View className="flex-row gap-2">
        <Button onPress={handleApprove} size="sm" variant="secondary">
          <Button.Label>Approve</Button.Label>
        </Button>
        <Button onPress={handleReject} size="sm" variant="danger-soft">
          <Button.Label>Reject</Button.Label>
        </Button>
      </View>
    </View>
  )
}
