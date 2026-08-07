import { existsSync, readFileSync } from "node:fs"
import { resolve } from "node:path"
import { defineConfig, devices } from "@playwright/test"
import { parse } from "dotenv"

type DotEnvValues = Record<string, string>

function readDotEnvFile(path: string): DotEnvValues {
  if (!existsSync(path)) {
    return {}
  }

  return parse(readFileSync(path))
}

function getRequiredValue(value: string | undefined, name: string): string {
  if (!value) {
    throw new Error(`${name} must be configured before running Playwright`)
  }

  return value
}

function getRequiredPort(value: string | undefined, name: string): number {
  const port = Number(getRequiredValue(value, name))

  if (!Number.isInteger(port) || port <= 0) {
    throw new Error(`${name} must be a positive integer`)
  }

  return port
}

function envAssignment(key: string, value: string | number): string {
  return `${key}=${value}`
}

const apiDotEnv = readDotEnvFile(resolve("apps/api/.env"))
const webDotEnv = readDotEnvFile(resolve("apps/web/.env"))
const shouldStartServers = process.env.PLAYWRIGHT_SKIP_WEBSERVER !== "true"

const apiPort = getRequiredPort(
  process.env.PLAYWRIGHT_RESOLVED_API_PORT ??
    process.env.PLAYWRIGHT_API_PORT ??
    process.env.PORT ??
    apiDotEnv.PORT,
  "PLAYWRIGHT_API_PORT or apps/api/.env PORT"
)
const webPort = getRequiredPort(
  process.env.PLAYWRIGHT_RESOLVED_WEB_PORT ??
    process.env.PLAYWRIGHT_WEB_PORT ??
    process.env.VITE_PORT ??
    webDotEnv.VITE_PORT,
  "PLAYWRIGHT_WEB_PORT or apps/web/.env VITE_PORT"
)

const apiBaseUrl =
  process.env.PLAYWRIGHT_API_URL ?? `http://localhost:${apiPort}`
const webBaseUrl =
  process.env.PLAYWRIGHT_BASE_URL ?? `http://localhost:${webPort}`

const braveExecutablePath =
  process.env.PLAYWRIGHT_BRAVE_EXECUTABLE_PATH ??
  process.env.PLAYWRIGHT_BROWSER_EXECUTABLE_PATH

const browserUse = braveExecutablePath
  ? {
      ...devices["Desktop Chrome"],
      launchOptions: {
        executablePath: braveExecutablePath,
      },
    }
  : devices["Desktop Chrome"]

const localApiEnv = [
  envAssignment("PORT", apiPort),
  envAssignment("BETTER_AUTH_URL", apiBaseUrl),
  envAssignment("CORS_ORIGIN", webBaseUrl),
].join(" ")

const localWebEnv = [
  envAssignment("VITE_PORT", webPort),
  envAssignment("VITE_API_URL", apiBaseUrl),
].join(" ")

export default defineConfig({
  expect: {
    timeout: 5000,
  },
  fullyParallel: true,
  metadata: {
    apiBaseUrl,
    webBaseUrl,
  },
  outputDir: ".playwright-output/test-results",
  projects: [
    {
      name: braveExecutablePath ? "brave" : "chromium",
      use: browserUse,
    },
  ],
  reporter: [
    ["list"],
    ["html", { open: "never", outputFolder: ".playwright-output/report" }],
  ],
  retries: process.env.CI ? 2 : 0,
  testDir: "./tests/e2e",
  use: {
    baseURL: webBaseUrl,
    video: "on",
  },
  webServer: shouldStartServers
    ? [
        {
          command: `${localApiEnv} bun run --cwd apps/api src/main.ts`,
          reuseExistingServer: false,
          timeout: 120_000,
          url: `${apiBaseUrl}/api/trpc/app.privateData?batch=1&input=%7B%7D`,
        },
        {
          command: `${localWebEnv} bun run --cwd apps/web dev`,
          reuseExistingServer: false,
          timeout: 120_000,
          url: webBaseUrl,
        },
      ]
    : undefined,
  workers: 1,
})
