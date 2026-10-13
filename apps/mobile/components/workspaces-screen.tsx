import { useQuery } from "@tanstack/react-query"
import { Link } from "expo-router"
import { Button, Spinner } from "heroui-native"
import { useCallback } from "react"
import { Pressable, Text, View } from "react-native"
import { authClient } from "@/lib/auth-client"
import { hiddenNote, mobileWorkspaces } from "@/lib/workspace-filter"
import { trpc } from "@/utils/trpc"

export function WorkspacesScreen() {
  const { data, error, isPending } = useQuery(
    trpc.workspaces.list.queryOptions()
  )

  const handleSignOut = useCallback(async () => {
    await authClient.signOut()
  }, [])

  const { hidden, visible } = mobileWorkspaces(data ?? [])
  const note = hiddenNote(hidden)

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

      {data && visible.length === 0 ? (
        <View className="gap-1">
          <Text className="font-geist text-base text-foreground">
            {hidden > 0
              ? "Nothing is shared with mobile."
              : "Open your first workspace."}
          </Text>
          <Text className="font-geist text-muted text-sm">
            {hidden > 0
              ? "Enable mobile supervision for a workspace in desktop Settings."
              : "Create a workspace on desktop to supervise it here."}
          </Text>
        </View>
      ) : null}

      {visible.map((workspace) => (
        <Link
          asChild
          href={{
            params: { workspaceId: workspace.id },
            pathname: "/workspace/[workspaceId]",
          }}
          key={workspace.id}
        >
          <Pressable className="gap-1 py-3">
            <Text className="font-geist text-base text-foreground">
              {workspace.name}
            </Text>
            <Text className="font-geist text-muted text-xs">
              {workspace.localPath}
            </Text>
          </Pressable>
        </Link>
      ))}

      {note ? (
        <Text className="font-geist text-muted text-xs">{note}</Text>
      ) : null}
    </View>
  )
}
