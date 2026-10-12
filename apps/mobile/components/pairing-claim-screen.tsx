import { useMutation } from "@tanstack/react-query"
import { Button, Spinner } from "heroui-native"
import { useCallback, useState } from "react"
import { Text, View } from "react-native"
import { trpc } from "@/utils/trpc"

interface PairingClaimScreenProps {
  token: string
}

export function PairingClaimScreen({ token }: PairingClaimScreenProps) {
  const [paired, setPaired] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const claim = useMutation(
    trpc.pairing.claim.mutationOptions({
      onError: (mutationError) => {
        setError(mutationError.message)
      },
      onSuccess: () => {
        setPaired(true)
      },
    })
  )

  const handleClaim = useCallback(() => {
    setError(null)
    claim.mutate({ token })
  }, [claim, token])

  return (
    <View className="flex-1 justify-center gap-4 bg-background px-6">
      <Text className="font-geist-semibold text-2xl text-foreground">
        Pair this device
      </Text>

      {paired ? (
        <>
          <Text className="font-geist text-base text-foreground">
            This device is paired.
          </Text>
          <Text className="font-geist text-muted text-sm">
            It can now supervise the paired workspace.
          </Text>
        </>
      ) : (
        <>
          <Text className="font-geist text-muted text-sm">
            Confirm to pair this phone with the desktop that showed the code.
          </Text>
          {error ? (
            <Text className="font-geist text-danger text-sm">{error}</Text>
          ) : null}
          <Button
            isDisabled={claim.isPending}
            onPress={handleClaim}
            variant="primary"
          >
            {claim.isPending ? (
              <Spinner />
            ) : (
              <Button.Label>Pair device</Button.Label>
            )}
          </Button>
        </>
      )}
    </View>
  )
}
