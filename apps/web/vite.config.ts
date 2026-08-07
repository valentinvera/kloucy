import babel from "@rolldown/plugin-babel"
import tailwindcss from "@tailwindcss/vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact, { reactCompilerPreset } from "@vitejs/plugin-react"
import { defineConfig, loadEnv } from "vite"

function getRequiredEnv(
  env: Record<string, string>,
  key: "VITE_API_URL" | "VITE_HOST" | "VITE_PORT"
): string {
  const value = env[key]

  if (!value) {
    throw new Error(`${key} must be configured in apps/web/.env`)
  }

  return value
}

function getRequiredPort(env: Record<string, string>): number {
  const port = Number(getRequiredEnv(env, "VITE_PORT"))

  if (!Number.isInteger(port) || port <= 0) {
    throw new Error("VITE_PORT must be a positive integer")
  }

  return port
}

const config = defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "")
  const apiUrl = getRequiredEnv(env, "VITE_API_URL")
  const host = getRequiredEnv(env, "VITE_HOST")
  const port = getRequiredPort(env)

  const secureValue = env.VITE_PROXY_SECURE === "true"

  const cookieDomainRewriteValue =
    env.VITE_PROXY_COOKIE_DOMAIN_REWRITE === "false"
      ? false
      : env.VITE_PROXY_COOKIE_DOMAIN_REWRITE || "localhost"

  return {
    plugins: [
      tailwindcss({
        optimize: true,
      }),
      tanstackStart({
        sitemap: {
          enabled: true,
          host: "https://.vercel.app",
          outputPath: "./public/sitemap.xml",
        },
      }),
      viteReact(),
      babel({
        presets: [reactCompilerPreset()],
      }),
    ],
    preview: {
      host,
      port,
    },
    resolve: {
      tsconfigPaths: true,
    },
    server: {
      host,
      port,
      proxy: {
        "/api": {
          autoRewrite: true,
          changeOrigin: true,
          cookieDomainRewrite: cookieDomainRewriteValue,
          secure: secureValue,
          target: apiUrl,
          timeout: 30_000,
        },
      },
      watch: {
        interval: 1000,
        usePolling: true,
      },
    },
  }
})

export default config
