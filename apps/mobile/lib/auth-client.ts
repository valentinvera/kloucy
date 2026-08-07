import { expoClient } from "@better-auth/expo/client"
import { env } from "@kloucy/env/mobile"
import { createAuthClient } from "better-auth/react"
import { getItem, setItem } from "expo-secure-store"

export const authClient = createAuthClient({
  baseURL: env.EXPO_PUBLIC_API_URL,
  plugins: [
    expoClient({
      scheme: "kloucy",
      storage: {
        getItem,
        setItem,
      },
      storagePrefix: "kloucy",
    }),
  ],
})
