import { existsSync, readdirSync, statSync } from "node:fs"
import { extname, join, relative, resolve, sep } from "node:path"
import { $, file, serve } from "bun"

interface Artifact {
  href: string
  kind: "brand" | "screenshot" | "video" | "report" | "other"
  name: string
  size: string
}

const artifactsDir = resolve(".playwright-output")
const host = process.env.PLAYWRIGHT_ARTIFACTS_HOST ?? "0.0.0.0"
const port = Number(process.env.PLAYWRIGHT_ARTIFACTS_PORT ?? 9324)
const LEADING_SLASHES_PATTERN = /^\/+/
const CONTENT_TYPES: Record<string, string> = {
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml; charset=utf-8",
  ".webm": "video/webm",
  ".zip": "application/zip",
}

function assertValidPort(value: number): void {
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error("PLAYWRIGHT_ARTIFACTS_PORT must be a positive integer")
  }
}

function getKind(path: string): Artifact["kind"] {
  if (path.startsWith(`${artifactsDir}${sep}brand${sep}`)) {
    return "brand"
  }

  const extension = extname(path)

  if (extension === ".webm") {
    return "video"
  }

  if (extension === ".png") {
    return "screenshot"
  }

  if (extension === ".svg") {
    return "brand"
  }

  if (path.endsWith("report/index.html")) {
    return "report"
  }

  return "other"
}

function getSize(bytes: number): string {
  const kib = bytes / 1024

  if (kib < 1024) {
    return `${Math.round(kib)} KB`
  }

  return `${(kib / 1024).toFixed(1)} MB`
}

function getArtifacts(dir: string): Artifact[] {
  const artifacts: Artifact[] = []

  if (!existsSync(dir)) {
    return artifacts
  }

  function visit(currentDir: string): void {
    for (const entry of readdirSync(currentDir)) {
      const path = join(currentDir, entry)
      const stats = statSync(path)

      if (stats.isDirectory()) {
        visit(path)
        continue
      }

      const kind = getKind(path)

      if (kind === "other") {
        continue
      }

      const href = `/${relative(artifactsDir, path).split(sep).join("/")}`

      artifacts.push({
        href,
        kind,
        name: href.slice(1),
        size: getSize(stats.size),
      })
    }
  }

  visit(dir)

  return artifacts.sort((first, second) =>
    `${first.kind}:${first.name}`.localeCompare(`${second.kind}:${second.name}`)
  )
}

function getContentType(path: string): string {
  return CONTENT_TYPES[extname(path)] ?? "application/octet-stream"
}

function getIndexHtml(artifacts: Artifact[]): string {
  const sections = ["brand", "video", "screenshot", "report"] as const

  const content = sections
    .map((section) => {
      const items = artifacts.filter((artifact) => artifact.kind === section)

      if (items.length === 0) {
        return ""
      }

      return `
        <section>
          <h2>${section}</h2>
          <ul>
            ${items
              .map(
                (artifact) => `
                  <li>
                    <a href="${artifact.href}">${artifact.name}</a>
                    <span>${artifact.size}</span>
                  </li>
                `
              )
              .join("")}
          </ul>
        </section>
      `
    })
    .join("")

  return `<!doctype html>
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>Kloucy Playwright Artifacts</title>
        <style>
          :root {
            color-scheme: dark;
            font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
          }
          body {
            background: #080808;
            color: #f5f5f5;
            margin: 0;
            padding: 24px;
          }
          main {
            margin: 0 auto;
            max-width: 960px;
          }
          h1 {
            font-size: 24px;
            margin: 0 0 8px;
          }
          h2 {
            border-bottom: 1px solid #333;
            font-size: 18px;
            margin-top: 28px;
            padding-bottom: 8px;
            text-transform: capitalize;
          }
          p {
            color: #b8b8b8;
            margin-top: 0;
          }
          ul {
            display: grid;
            gap: 8px;
            list-style: none;
            padding: 0;
          }
          li {
            align-items: center;
            border: 1px solid #262626;
            border-radius: 8px;
            display: flex;
            gap: 16px;
            justify-content: space-between;
            padding: 12px;
          }
          a {
            color: #8ec5ff;
            overflow-wrap: anywhere;
          }
          span {
            color: #a3a3a3;
            flex: 0 0 auto;
            font-size: 12px;
          }
          code {
            background: #181818;
            border-radius: 4px;
            padding: 2px 6px;
          }
        </style>
      </head>
      <body>
        <main>
          <h1>Kloucy Playwright Artifacts</h1>
          <p>Open brand assets, videos, screenshots, and the Playwright HTML report directly from your browser.</p>
          ${content || "<p>No Playwright artifacts found yet. Run <code>bun run test:e2e</code> first.</p>"}
        </main>
      </body>
    </html>`
}

async function getTailscaleIps(): Promise<string[]> {
  try {
    const output = await $`tailscale ip -4`.quiet().text()
    return output
      .split("\n")
      .map((value) => value.trim())
      .filter(Boolean)
  } catch {
    return []
  }
}

function getSafeFilePath(pathname: string): string | null {
  const decodedPath = decodeURIComponent(pathname)
  const relativePath = decodedPath.replace(LEADING_SLASHES_PATTERN, "")
  const filePath = resolve(artifactsDir, relativePath)

  if (
    !(filePath === artifactsDir || filePath.startsWith(`${artifactsDir}${sep}`))
  ) {
    return null
  }

  return filePath
}

assertValidPort(port)

const server = serve({
  fetch(request) {
    const url = new URL(request.url)

    if (url.pathname === "/") {
      return new Response(getIndexHtml(getArtifacts(artifactsDir)), {
        headers: {
          "content-type": "text/html; charset=utf-8",
        },
      })
    }

    const filePath = getSafeFilePath(url.pathname)

    if (!(filePath && existsSync(filePath))) {
      return new Response("Not found", { status: 404 })
    }

    const artifactFile = file(filePath)

    return new Response(artifactFile, {
      headers: {
        "content-type": getContentType(filePath),
      },
    })
  },
  hostname: host,
  port,
})

const tailscaleIps = await getTailscaleIps()

console.log(`Serving ${artifactsDir}`)
console.log(`Local: http://localhost:${server.port}`)

for (const ip of tailscaleIps) {
  console.log(`Tailscale: http://${ip}:${server.port}`)
}
