export const appName = "Kloucy"
export const docsRoute = "/docs"
export const docsImageRoute = "/og/docs"

const MARKDOWN_EXTENSION_REGEX = /\.md$/

export const gitConfig = {
  branch: "main",
  repo: "kloucy",
  user: "valentinvera",
}

export function encodeMarkdownUrl(slugs: string[], locale?: string) {
  const segments = [...slugs]
  if (segments.length === 0) {
    segments.push("index.md")
  } else {
    segments[segments.length - 1] += ".md"
  }

  return (
    "/" +
    [locale, ...docsRoute.split("/"), ...segments].filter(Boolean).join("/")
  )
}

/** @returns page slugs */
export function decodeMarkdownUrl(segments: string[]) {
  if (segments.length === 0) {
    return []
  }

  const out = [...segments]
  out[out.length - 1] = (out.at(-1) ?? "").replace(MARKDOWN_EXTENSION_REGEX, "")
  if (out.length === 1 && out[0] === "index") {
    out.pop()
  }
  return out
}
