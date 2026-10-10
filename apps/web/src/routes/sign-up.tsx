import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Button } from "@kloucy/ui/components/button"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router"
import { type ChangeEvent, type FormEvent, useCallback, useState } from "react"
import { AuthShell } from "@/components/auth-shell"
import { SignedInNotice } from "@/components/signed-in-notice"
import { authClient } from "@/lib/auth-client"

export const Route = createFileRoute("/sign-up")({
  component: SignUpComponent,
})

function SignUpComponent() {
  const navigate = useNavigate()
  const { data: session } = authClient.useSession()

  const [name, setName] = useState("")
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleNameChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setName(event.target.value)
    },
    []
  )

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

      const { error: signUpError } = await authClient.signUp.email({
        email,
        name,
        password,
      })

      if (signUpError) {
        setError(signUpError.message ?? "Could not create your account.")
        setIsSubmitting(false)
        return
      }

      await navigate({ to: "/" })
    },
    [email, name, navigate, password]
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
      description="Create an account to run and supervise local agents."
      footer={
        <p className="text-medium text-muted-foreground">
          Already have an account?{" "}
          <Link
            className="text-foreground underline underline-offset-4"
            to="/sign-in"
          >
            Sign in
          </Link>
        </p>
      }
      title="Create your account"
    >
      <form noValidate onSubmit={handleSubmit}>
        <FieldGroup>
          {error ? (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : null}
          <Field>
            <FieldLabel htmlFor="name">Name</FieldLabel>
            <Input
              autoComplete="name"
              id="name"
              onChange={handleNameChange}
              required
              type="text"
              value={name}
            />
          </Field>
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
              autoComplete="new-password"
              id="password"
              minLength={8}
              onChange={handlePasswordChange}
              required
              type="password"
              value={password}
            />
            <FieldDescription>Use at least 8 characters.</FieldDescription>
          </Field>
          <Button className="w-full" disabled={isSubmitting} type="submit">
            {isSubmitting ? "Creating account…" : "Create account"}
          </Button>
        </FieldGroup>
      </form>
    </AuthShell>
  )
}
