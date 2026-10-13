import { describe, expect, test } from "bun:test"
import {
  sessionCounts,
  summarizeCounts,
} from "../../apps/mobile/lib/session-counts"

describe("sessionCounts", () => {
  test("groups statuses and keeps the total", () => {
    const counts = sessionCounts([
      { status: "waiting" },
      { status: "waiting" },
      { status: "running" },
      { status: "ready" },
      { status: "done" },
      { status: "failed" },
      { status: "stopped" },
    ])

    expect(counts).toEqual({
      blocked: 2,
      done: 1,
      failed: 1,
      ready: 1,
      running: 1,
      total: 7,
    })
  })

  test("ignores statuses outside the map", () => {
    const counts = sessionCounts([{ status: "needs_setup" }])

    expect(counts.total).toBe(1)
    expect(counts.running).toBe(0)
  })

  test("counts nothing for an empty list", () => {
    expect(sessionCounts([]).total).toBe(0)
  })
})

describe("summarizeCounts", () => {
  test("says so when there is nothing", () => {
    expect(summarizeCounts(sessionCounts([]))).toBe("No sessions yet.")
  })

  test("puts blocked work first", () => {
    const line = summarizeCounts(
      sessionCounts([
        { status: "done" },
        { status: "waiting" },
        { status: "running" },
      ])
    )

    expect(line).toBe("1 waiting · 1 running · 1 done")
  })

  test("falls back when nothing maps", () => {
    expect(summarizeCounts(sessionCounts([{ status: "stopped" }]))).toBe(
      "Nothing running."
    )
  })
})
