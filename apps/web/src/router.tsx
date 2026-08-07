import { env } from "@kloucy/env/web"
import type { AppRouter } from "@kloucy/trpc/generated/server"
import { QueryCache, QueryClient } from "@tanstack/react-query"
import { createRouter as createTanStackRouter } from "@tanstack/react-router"
import { setupRouterSsrQueryIntegration } from "@tanstack/react-router-ssr-query"
import { createTRPCClient, httpBatchLink } from "@trpc/client"
import { createTRPCOptionsProxy } from "@trpc/tanstack-react-query"
import { toast } from "sonner"
import { routeTree } from "./routeTree.gen"
import { TRPCProvider } from "./utils/trpc"

const globalFetch = globalThis.fetch.bind(globalThis)

function createQueryClient() {
  return new QueryClient({
    defaultOptions: { queries: { staleTime: 60 * 1000 } },
    queryCache: new QueryCache({
      onError: (error, query) => {
        toast.error(error.message, {
          action: {
            label: "retry",
            onClick: () => {
              query.invalidate()
            },
          },
        })
      },
    }),
  })
}

const trpcClient = createTRPCClient<AppRouter>({
  links: [
    httpBatchLink({
      fetch(url, options) {
        return globalFetch(url, {
          ...options,
          credentials: "include",
        })
      },
      url: `${env.VITE_API_URL}/api/trpc`,
    }),
  ],
})

export const getRouter = () => {
  const queryClient = createQueryClient()
  const trpc = createTRPCOptionsProxy({
    client: trpcClient,
    queryClient,
  })

  const router = createTanStackRouter({
    context: { queryClient, trpc },
    defaultNotFoundComponent: () => <div>Not Found</div>,
    defaultPendingComponent: () => <div>Loading...</div>,
    defaultPreloadStaleTime: 0,
    routeTree,
    scrollRestoration: true,
    Wrap: ({ children }) => (
      <TRPCProvider queryClient={queryClient} trpcClient={trpcClient}>
        {children}
      </TRPCProvider>
    ),
  })

  setupRouterSsrQueryIntegration({
    queryClient,
    router,
  })

  return router
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof getRouter>
  }
}
