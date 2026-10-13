import { describe, expect, test } from "bun:test"
import { nextLogs } from "../../apps/mobile/lib/session-output"

describe("nextLogs", () => {
  test("appends new lines in order", () => {
    const first = [{ id: "1", message: "one", stream: "stdout" }]
    const second = [{ id: "2", message: "two", stream: "stderr" }]

    expect(nextLogs(first, second).map((log) => log.id)).toEqual(["1", "2"])
  })

  test("ignores lines the app already holds", () => {
    const held = [
      { id: "1", message: "one", stream: "stdout" },
      { id: "2", message: "two", stream: "stdout" },
    ]
    const repeated = [{ id: "2", message: "two", stream: "stdout" }]

    expect(nextLogs(held, repeated)).toEqual(held)
  })

  test("handles an empty history and an empty page", () => {
    expect(nextLogs([], [])).toEqual([])

    const line = { id: "1", message: "one", stream: "stdout" }

    expect(nextLogs([], [line])).toEqual([line])
  })
})
