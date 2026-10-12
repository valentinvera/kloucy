import {
  Button,
  FieldError,
  Input,
  Label,
  Spinner,
  TextField,
} from "heroui-native"
import { useCallback, useState } from "react"
import { Text, View } from "react-native"
import { authClient } from "@/lib/auth-client"

export default function Home() {
  const { data: session, isPending } = authClient.useSession()

  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  const handleSignIn = useCallback(async () => {
    setError(null)
    setIsSubmitting(true)

    const { error: signInError } = await authClient.signIn.email({
      email,
      password,
    })

    if (signInError) {
      setError(signInError.message ?? "Could not sign in.")
    }

    setIsSubmitting(false)
  }, [email, password])

  const handleSignOut = useCallback(async () => {
    await authClient.signOut()
  }, [])

  if (isPending) {
    return (
      <View className="flex-1 items-center justify-center bg-background">
        <Spinner />
      </View>
    )
  }

  if (session) {
    return (
      <View className="flex-1 justify-center gap-4 bg-background px-6">
        <Text className="font-geist-semibold text-2xl text-foreground">
          You are signed in
        </Text>
        <Text className="font-geist text-muted text-sm">
          {session.user.email}
        </Text>
        <Button onPress={handleSignOut} variant="secondary">
          <Button.Label>Sign out</Button.Label>
        </Button>
      </View>
    )
  }

  return (
    <View className="flex-1 justify-center gap-6 bg-background px-6">
      <Text className="font-geist-semibold text-2xl text-foreground">
        Sign in to Kloucy
      </Text>

      <TextField isInvalid={Boolean(error)} isRequired>
        <Label>Email</Label>
        <Input
          autoCapitalize="none"
          keyboardType="email-address"
          onChangeText={setEmail}
          placeholder="you@example.com"
          value={email}
        />
      </TextField>

      <TextField isInvalid={Boolean(error)} isRequired>
        <Label>Password</Label>
        <Input
          onChangeText={setPassword}
          placeholder="Password"
          secureTextEntry
          value={password}
        />
        {error ? <FieldError>{error}</FieldError> : null}
      </TextField>

      <Button
        isDisabled={isSubmitting}
        onPress={handleSignIn}
        variant="primary"
      >
        {isSubmitting ? <Spinner /> : <Button.Label>Sign in</Button.Label>}
      </Button>
    </View>
  )
}
