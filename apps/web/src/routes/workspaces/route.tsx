import { createFileRoute, Outlet } from "@tanstack/react-router"
import { AppShell } from "@/components/app-shell"

export const Route = createFileRoute("/workspaces")({
  component: WorkspacesLayout,
})

function WorkspacesLayout() {
  return (
    <AppShell>
      <Outlet />
    </AppShell>
  )
}
