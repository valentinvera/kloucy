import { describe, expect, test } from "bun:test"
import { nextActions } from "../../apps/mobile/lib/next-actions"

describe("nextActions", () => {
  test("says nothing needs you when the workspace is quiet", () => {
    const actions = nextActions({ reviews: [], sessions: [] })

    expect(actions).toHaveLength(1)
    expect(actions[0]?.title).toBe("Nothing needs you")
  })

  test("lists at most three actions, most blocking first", () => {
    const actions = nextActions({
      reviews: [{ action: "pending", id: "r1" }],
      sessions: [
        { id: "s1", prompt: "Apply the migration", status: "waiting" },
        { id: "s2", prompt: null, status: "failed" },
        { id: "s3", prompt: null, status: "ready" },
        { id: "s4", prompt: null, status: "running" },
      ],
    })

    expect(actions).toHaveLength(3)
    expect(actions.map((action) => action.title)).toEqual([
      "Answer 1 waiting session",
      "Review 1 change",
      "Retry 1 failed session",
    ])
    expect(actions[0]?.detail).toBe(
      "Apply the migration needs input or approval."
    )
  })

  test("pluralizes counts", () => {
    const actions = nextActions({
      reviews: [],
      sessions: [
        { id: "s1", prompt: null, status: "ready" },
        { id: "s2", prompt: null, status: "ready" },
      ],
    })

    expect(actions[0]?.title).toBe("Start 2 ready sessions")
  })

  test("ignores reviews that are already decided", () => {
    const actions = nextActions({
      reviews: [{ action: "approved", id: "r1" }],
      sessions: [{ id: "s1", prompt: null, status: "running" }],
    })

    expect(actions.map((action) => action.title)).toEqual(["1 session running"])
  })
})
