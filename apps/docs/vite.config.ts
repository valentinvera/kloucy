import tailwindcss from "@tailwindcss/vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import react from "@vitejs/plugin-react"
import { fumadocsMdx } from "fumadocs-mdx/vite"
import { nitro } from "nitro/vite"
import { defineConfig, loadEnv } from "vite"

const getRequiredEnv = (
  env: Record<string, string>,
  key: "DOCS_HOST" | "DOCS_PORT"
): string => {
  const value = env[key]

  if (!value) {
    throw new Error(`${key} must be configured in apps/docs/.env`)
  }

  return value
}

const getRequiredPort = (env: Record<string, string>): number => {
  const port = Number(getRequiredEnv(env, "DOCS_PORT"))

  if (!Number.isInteger(port) || port <= 0) {
    throw new Error("DOCS_PORT must be a positive integer")
  }

  return port
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "")
  const host = getRequiredEnv(env, "DOCS_HOST")
  const port = getRequiredPort(env)

  return {
    plugins: [
      fumadocsMdx(),
      tailwindcss(),
      tanstackStart({
        pages: [
          {
            path: "/docs",
          },
          {
            path: "/api/search",
          },
          {
            path: "llms-full.txt",
          },
          {
            path: "llms.txt",
          },
        ],
        spa: {
          enabled: true,
          prerender: {
            crawlLinks: true,
            enabled: true,
          },
        },
      }),
      react(),
      // please see https://tanstack.com/start/latest/docs/framework/react/guide/hosting#nitro for guides on hosting
      nitro(),
    ],
    preview: {
      host,
      port,
    },
    resolve: {
      alias: {
        tslib: "tslib/tslib.es6.js",
      },
      tsconfigPaths: true,
    },
    server: {
      host,
      port,
    },
  }
})
