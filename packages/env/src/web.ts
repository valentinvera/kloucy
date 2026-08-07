import { createEnv } from "@t3-oss/env-core"
import { z } from "zod"

type ViteRuntimeEnv = Record<string, string | boolean | undefined>

export const env = createEnv({
  client: {
    VITE_API_URL: z.url(),
    VITE_HOST: z.string().min(1),
    VITE_PORT: z.coerce.number().int().positive(),
  },
  clientPrefix: "VITE_",
  emptyStringAsUndefined: true,
  runtimeEnv: (import.meta as ImportMeta & { env: ViteRuntimeEnv }).env,
})
