import { expo } from "@better-auth/expo"
import { createDb } from "@kloucy/db"
import {
  account,
  accountRelations,
  session,
  sessionRelations,
  user,
  userRelations,
  verification,
} from "@kloucy/db/schema/auth"
import { env, getCorsOrigins } from "@kloucy/env/api"
import { betterAuth } from "better-auth"
import { drizzleAdapter } from "better-auth/adapters/drizzle"

const authSchema = {
  account,
  accountRelations,
  session,
  sessionRelations,
  user,
  userRelations,
  verification,
}

export function createAuth() {
  const db = createDb()

  // A `Secure` cookie is only sent back over a secure context, and plain HTTP
  // only counts as one on localhost. A local-first setup reaches the API over
  // HTTP on a LAN or Tailscale address, where a `Secure` cookie is stored but
  // never returned, so the session would silently not stick. HTTPS deployments
  // keep the strict attributes.
  const isHttps = env.BETTER_AUTH_URL.startsWith("https")

  return betterAuth({
    advanced: {
      defaultCookieAttributes: isHttps
        ? {
            httpOnly: true,
            sameSite: "none",
            secure: true,
          }
        : {
            httpOnly: true,
            sameSite: "lax",
            secure: false,
          },
    },
    baseURL: env.BETTER_AUTH_URL,
    database: drizzleAdapter(db, {
      provider: "pg",

      schema: authSchema,
    }),
    emailAndPassword: {
      enabled: true,
    },
    plugins: [expo()],
    secret: env.BETTER_AUTH_SECRET,
    trustedOrigins: [...getCorsOrigins(), "kloucy://", "exp://"],
  })
}

export const auth = createAuth()
