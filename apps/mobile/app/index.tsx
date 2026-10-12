import { Spinner } from "heroui-native"
import { View } from "react-native"
import { SignInScreen } from "@/components/sign-in-screen"
import { WorkspacesScreen } from "@/components/workspaces-screen"
import { authClient } from "@/lib/auth-client"

export default function Home() {
  const { data: session, isPending } = authClient.useSession()

  if (isPending) {
    return (
      <View className="flex-1 items-center justify-center bg-background">
        <Spinner />
      </View>
    )
  }

  return session ? <WorkspacesScreen /> : <SignInScreen />
}
