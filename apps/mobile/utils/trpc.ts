import { env } from "@kloucy/env/mobile"
import type { AppRouter } from "@kloucy/trpc/generated/server"
import { QueryClient } from "@tanstack/react-query"
import { createTRPCClient, httpBatchLink } from "@trpc/client"
import { createTRPCOptionsProxy } from "@trpc/tanstack-react-query"

import { authClient } from "@/lib/auth-client"

function withAuthCookie(headers: HeadersInit | undefined): Headers {
  const nextHeaders = new Headers(headers)
  const cookie = authClient.getCookie()

  if (cookie) {
    nextHeaders.set("cookie", cookie)
  }

  return nextHeaders
}

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000,
    },
  },
})

export const trpcClient = createTRPCClient<AppRouter>({
  links: [
    httpBatchLink({
      fetch(url, options) {
        return globalThis.fetch(url, {
          ...options,
          credentials: "include",
          headers: withAuthCookie(options?.headers),
        })
      },
      url: `${env.EXPO_PUBLIC_API_URL}/api/trpc`,
    }),
  ],
})

export const trpc = createTRPCOptionsProxy({
  client: trpcClient,
  queryClient,
})
