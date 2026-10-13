import { describe, expect, test } from "bun:test"
import {
  detectAgents,
  executableToTest,
  KNOWN_AGENTS,
} from "../../packages/trpc/src/lib/agent-registry"

describe("detectAgents", () => {
  test("returns the agents that resolve on the path, in catalog order", () => {
    const installed = new Set(["/usr/local/bin/codex", "/opt/bin/aider"])
    const found = detectAgents(
      "/usr/bin:/usr/local/bin:/opt/bin",
      (candidate) => installed.has(candidate)
    )

    expect(found).toEqual([
      {
        executablePath: "/usr/local/bin/codex",
        name: "Codex",
        provider: "codex",
      },
      { executablePath: "/opt/bin/aider", name: "Aider", provider: "aider" },
    ])
  })

  test("takes the first match in path order", () => {
    const found = detectAgents("/one:/two", (candidate) =>
      candidate.endsWith("/claude")
    )

    expect(found).toEqual([
      {
        executablePath: "/one/claude",
        name: "Claude Code",
        provider: "claude-code",
      },
    ])
  })

  test("reports each catalog entry that resolves", () => {
    const found = detectAgents("/bin", () => true)

    expect(found.map((agent) => agent.provider)).toEqual(
      KNOWN_AGENTS.map((agent) => agent.provider)
    )
  })

  test("finds nothing on an empty path", () => {
    expect(detectAgents("", () => true)).toEqual([])
  })
})

describe("executableToTest", () => {
  test("prefers the executable path", () => {
    expect(
      executableToTest({
        commandTemplate: "codex exec",
        executablePath: "/bin/codex",
      })
    ).toBe("/bin/codex")
  })

  test("falls back to the first word of the command", () => {
    expect(
      executableToTest({
        commandTemplate: "  codex exec --full-auto ",
        executablePath: null,
      })
    ).toBe("codex")
  })

  test("has nothing to test without either", () => {
    expect(
      executableToTest({ commandTemplate: null, executablePath: null })
    ).toBeNull()
    expect(
      executableToTest({ commandTemplate: "   ", executablePath: null })
    ).toBeNull()
  })
})
