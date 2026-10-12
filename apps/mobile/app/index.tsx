import { Spinner } from "heroui-native"
import { View } from "react-native"
import { PairingClaimScreen } from "@/components/pairing-claim-screen"
import { SignInScreen } from "@/components/sign-in-screen"
import { WorkspacesScreen } from "@/components/workspaces-screen"
import { authClient } from "@/lib/auth-client"
import { usePairingToken } from "@/lib/use-pairing-token"

export default function Home() {
  const { data: session, isPending } = authClient.useSession()
  const pairingToken = usePairingToken()

  if (isPending) {
    return (
      <View className="flex-1 items-center justify-center bg-background">
        <Spinner />
      </View>
    )
  }

  if (session && pairingToken) {
    return <PairingClaimScreen token={pairingToken} />
  }

  return session ? <WorkspacesScreen /> : <SignInScreen />
}
