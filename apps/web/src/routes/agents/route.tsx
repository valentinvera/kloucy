import { createFileRoute, Outlet } from "@tanstack/react-router"
import { AppShell } from "@/components/app-shell"

export const Route = createFileRoute("/agents")({
  component: AgentsLayout,
})

function AgentsLayout() {
  return (
    <AppShell>
      <Outlet />
    </AppShell>
  )
}
