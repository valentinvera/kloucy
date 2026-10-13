import { delimiter } from "node:path"

/** An agent CLI Kloucy looks for, mirroring the desktop's catalog. */
interface KnownAgent {
  binary: string
  name: string
  provider: string
}

/** Catalog of agent CLIs, checked in this order. */
export const KNOWN_AGENTS: KnownAgent[] = [
  { binary: "claude", name: "Claude Code", provider: "claude-code" },
  { binary: "codex", name: "Codex", provider: "codex" },
  { binary: "cursor-agent", name: "Cursor Agent", provider: "cursor-agent" },
  { binary: "gemini", name: "Gemini CLI", provider: "gemini" },
  { binary: "goose", name: "Goose", provider: "goose" },
  { binary: "aider", name: "Aider", provider: "aider" },
  { binary: "opencode", name: "OpenCode", provider: "opencode" },
]

export interface DetectedAgent {
  executablePath: string
  name: string
  provider: string
}

function findOnPath(
  binary: string,
  path: string,
  isExecutable: (candidate: string) => boolean
): string | undefined {
  for (const directory of path.split(delimiter)) {
    if (directory.length === 0) {
      continue
    }

    const candidate = `${directory}/${binary}`

    if (isExecutable(candidate)) {
      return candidate
    }
  }
}

/**
 * Agent CLIs present on `path`. Resolving executables is all this does; nothing
 * found is ever run.
 */
export function detectAgents(
  path: string,
  isExecutable: (candidate: string) => boolean
): DetectedAgent[] {
  return KNOWN_AGENTS.flatMap((known) => {
    const executablePath = findOnPath(known.binary, path, isExecutable)

    if (!executablePath) {
      return []
    }

    return [{ executablePath, name: known.name, provider: known.provider }]
  })
}

const WHITESPACE = /\s+/

/**
 * The executable a test should run: the agent's own path when it has one, and
 * otherwise the first word of its command.
 */
export function executableToTest(input: {
  commandTemplate: string | null
  executablePath: string | null
}): string | null {
  if (input.executablePath) {
    return input.executablePath
  }

  const [first] = input.commandTemplate?.trim().split(WHITESPACE) ?? []

  return first && first.length > 0 ? first : null
}
