import { useNavigate } from "@tanstack/react-router"
import { useEffect } from "react"
import { authClient } from "@/lib/auth-client"

export function useRequireSession() {
  const navigate = useNavigate()
  const { data: session, isPending } = authClient.useSession()

  useEffect(() => {
    if (!(isPending || session)) {
      navigate({ to: "/sign-in" }).catch(() => undefined)
    }
  }, [isPending, navigate, session])

  return { isPending, session }
}
