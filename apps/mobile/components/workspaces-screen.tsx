import { useQuery } from "@tanstack/react-query"
import { Button, Spinner } from "heroui-native"
import { useCallback } from "react"
import { Text, View } from "react-native"
import { authClient } from "@/lib/auth-client"
import { trpc } from "@/utils/trpc"

export function WorkspacesScreen() {
  const { data, error, isPending } = useQuery(
    trpc.workspaces.list.queryOptions()
  )

  const handleSignOut = useCallback(async () => {
    await authClient.signOut()
  }, [])

  return (
    <View className="flex-1 gap-4 bg-background px-6 pt-16">
      <View className="flex-row items-center justify-between">
        <Text className="font-geist-semibold text-2xl text-foreground">
          Workspaces
        </Text>
        <Button onPress={handleSignOut} size="sm" variant="ghost">
          <Button.Label>Sign out</Button.Label>
        </Button>
      </View>

      {isPending ? <Spinner /> : null}

      {error ? (
        <Text className="font-geist text-danger text-sm">{error.message}</Text>
      ) : null}

      {data && data.length === 0 ? (
        <View className="gap-1">
          <Text className="font-geist text-base text-foreground">
            Open your first workspace.
          </Text>
          <Text className="font-geist text-muted text-sm">
            Create a workspace on desktop to supervise it here.
          </Text>
        </View>
      ) : null}

      {data?.map((workspace) => (
        <View className="gap-1" key={workspace.id}>
          <Text className="font-geist text-base text-foreground">
            {workspace.name}
          </Text>
          <Text className="font-geist text-muted text-xs">
            {workspace.localPath}
          </Text>
        </View>
      ))}
    </View>
  )
}
