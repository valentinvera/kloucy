import { Button } from "@kloucy/ui/components/button"
import { useNavigate } from "@tanstack/react-router"
import { useCallback, useState } from "react"
import { authClient } from "@/lib/auth-client"

interface SignedInNoticeProps {
  email: string
}

export function SignedInNotice({ email }: SignedInNoticeProps) {
  const navigate = useNavigate()
  const [isSigningOut, setIsSigningOut] = useState(false)

  const handleSignOut = useCallback(async () => {
    setIsSigningOut(true)
    await authClient.signOut()
    await navigate({ to: "/sign-in" })
  }, [navigate])

  return (
    <div className="flex flex-col gap-4">
      <p className="text-medium text-muted-foreground">
        You are signed in as <span className="text-foreground">{email}</span>.
      </p>
      <Button
        className="w-fit"
        disabled={isSigningOut}
        onClick={handleSignOut}
        variant="outline"
      >
        {isSigningOut ? "Signing out…" : "Sign out"}
      </Button>
    </div>
  )
}
