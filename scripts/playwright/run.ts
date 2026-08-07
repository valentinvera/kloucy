import { spawn } from "node:child_process"
import { existsSync, readFileSync } from "node:fs"
import net from "node:net"
import { resolve } from "node:path"
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

function isPortAvailable(port: number): Promise<boolean> {
  return new Promise((resolvePortAvailability) => {
    const server = net.createServer()

    server.once("error", () => resolvePortAvailability(false))
    server.once("listening", () => {
      server.close(() => resolvePortAvailability(true))
    })
    server.listen(port, "127.0.0.1")
  })
}

async function resolveAvailablePort(port: number): Promise<number> {
  if (await isPortAvailable(port)) {
    return port
  }

  const nextPort = port + 1

  if (await isPortAvailable(nextPort)) {
    return nextPort
  }

  throw new Error(`Neither ${port} nor ${nextPort} is available`)
}

const apiDotEnv = readDotEnvFile(resolve("apps/api/.env"))
const webDotEnv = readDotEnvFile(resolve("apps/web/.env"))
const shouldStartServers = process.env.PLAYWRIGHT_SKIP_WEBSERVER !== "true"

const requestedApiPort = getRequiredPort(
  process.env.PLAYWRIGHT_API_PORT ?? process.env.PORT ?? apiDotEnv.PORT,
  "PLAYWRIGHT_API_PORT or apps/api/.env PORT"
)
const requestedWebPort = getRequiredPort(
  process.env.PLAYWRIGHT_WEB_PORT ??
    process.env.VITE_PORT ??
    webDotEnv.VITE_PORT,
  "PLAYWRIGHT_WEB_PORT or apps/web/.env VITE_PORT"
)

const apiPort = shouldStartServers
  ? await resolveAvailablePort(requestedApiPort)
  : requestedApiPort
const webPort = shouldStartServers
  ? await resolveAvailablePort(requestedWebPort)
  : requestedWebPort

const child = spawn("bunx", ["--bun", "playwright", ...process.argv.slice(2)], {
  env: {
    ...process.env,
    PLAYWRIGHT_RESOLVED_API_PORT: String(apiPort),
    PLAYWRIGHT_RESOLVED_WEB_PORT: String(webPort),
  },
  stdio: "inherit",
})

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 1)
})
