import { Button } from "@kloucy/ui/components/button"
import {
  createFileRoute,
  Link,
  Outlet,
  useNavigate,
} from "@tanstack/react-router"
import { useCallback, useState } from "react"
import { useRequireSession } from "@/hooks/use-require-session"
import { authClient } from "@/lib/auth-client"

export const Route = createFileRoute("/workspaces")({
  component: WorkspacesLayout,
})

function WorkspacesLayout() {
  const { isPending, session } = useRequireSession()
  const navigate = useNavigate()
  const [isSigningOut, setIsSigningOut] = useState(false)

  const handleSignOut = useCallback(async () => {
    setIsSigningOut(true)
    await authClient.signOut()
    await navigate({ to: "/sign-in" })
  }, [navigate])

  if (isPending || !session) {
    return (
      <main className="grid min-h-svh place-items-center px-6">
        <p className="text-medium text-muted-foreground">
          Checking your session…
        </p>
      </main>
    )
  }

  return (
    <div className="flex min-h-svh flex-col">
      <header className="flex items-center justify-between border-border/60 border-b px-6 py-3">
        <Link className="font-logo text-section" to="/workspaces">
          Kloucy
        </Link>
        <div className="flex items-center gap-3">
          <span className="hidden text-caption text-muted-foreground sm:inline">
            {session.user.email}
          </span>
          <Button
            disabled={isSigningOut}
            onClick={handleSignOut}
            size="sm"
            variant="ghost"
          >
            {isSigningOut ? "Signing out…" : "Sign out"}
          </Button>
        </div>
      </header>
      <Outlet />
    </div>
  )
}
