import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Button } from "@kloucy/ui/components/button"
import { Field, FieldGroup, FieldLabel } from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import { type ChangeEvent, type FormEvent, useCallback, useState } from "react"
import { AuthShell } from "@/components/auth-shell"
import { SignedInNotice } from "@/components/signed-in-notice"
import { authClient } from "@/lib/auth-client"

export const Route = createFileRoute("/sign-in")({
  component: SignInComponent,
})

function SignInComponent() {
  const navigate = useNavigate()
  const { data: session } = authClient.useSession()

  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleEmailChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setEmail(event.target.value)
    },
    []
  )

  const handlePasswordChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setPassword(event.target.value)
    },
    []
  )

  const handleSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      setError(null)
      setIsSubmitting(true)

      const { error: signInError } = await authClient.signIn.email({
        email,
        password,
      })

      if (signInError) {
        setError(
          signInError.message ??
            "Could not sign in. Check your email and password."
        )
        setIsSubmitting(false)
        return
      }

      await navigate({ to: "/" })
    },
    [email, password, navigate]
  )

  if (session) {
    return (
      <AuthShell
        description="Continue to Kloucy, or sign out of this device."
        title="You are signed in"
      >
        <SignedInNotice email={session.user.email} />
      </AuthShell>
    )
  }

  return (
    <AuthShell
      description="Enter your credentials to continue to your workspaces."
      footer={
        <p className="text-medium text-muted-foreground">
          Need an account?{" "}
          <Link
            className="text-foreground underline underline-offset-4"
            to="/sign-up"
          >
            Create one
          </Link>
        </p>
      }
      title="Sign in"
    >
      <form noValidate onSubmit={handleSubmit}>
        <FieldGroup>
          {error ? (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
          <Field>
            <FieldLabel htmlFor="email">Email</FieldLabel>
            <Input
              autoComplete="email"
              id="email"
              onChange={handleEmailChange}
              required
              type="email"
              value={email}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="password">Password</FieldLabel>
            <Input
              autoComplete="current-password"
              id="password"
              onChange={handlePasswordChange}
              required
              type="password"
              value={password}
            />
          </Field>
          <Button className="w-full" disabled={isSubmitting} type="submit">
            {isSubmitting ? "Signing in…" : "Sign in"}
          </Button>
        </FieldGroup>
      </form>
    </AuthShell>
  )
}
