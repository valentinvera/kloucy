import { describe, expect, test } from "bun:test"
import {
  buildWeeklyReview,
  WEEK_MS,
  type WeeklyActivity,
} from "../../packages/trpc/src/lib/weekly-review"

const now = new Date("2026-10-13T12:00:00.000Z")
const today = new Date("2026-10-13T09:00:00.000Z")
const lastWeek = new Date(now.getTime() - WEEK_MS - 60_000)

function empty(): WeeklyActivity {
  return { reviews: [], sessions: [] }
}

describe("buildWeeklyReview", () => {
  test("says nothing needs you for an empty week", () => {
    const draft = buildWeeklyReview(empty(), now)

    expect(draft.summary).toBe(
      "0 done, 0 waiting, 0 failed, 0 rejected this week."
    )
    expect(draft.shipped).toEqual([])
    expect(draft.nextActions).toEqual(["Start a task from the inbox"])
  })

  test("lists what shipped during the week", () => {
    const draft = buildWeeklyReview(
      {
        reviews: [],
        sessions: [
          {
            createdAt: today,
            endedAt: today,
            prompt: "Add the readiness snapshot",
            status: "done",
          },
          {
            createdAt: lastWeek,
            endedAt: lastWeek,
            prompt: "Old work",
            status: "done",
          },
        ],
      },
      now
    )

    expect(draft.shipped).toEqual(["Add the readiness snapshot"])
    expect(draft.nextActions).toEqual(["Nothing needs you"])
  })

  test("reports waiting sessions as blockers even when they are older", () => {
    const draft = buildWeeklyReview(
      {
        reviews: [],
        sessions: [
          {
            createdAt: lastWeek,
            endedAt: null,
            prompt: "Apply the migration",
            status: "waiting",
          },
        ],
      },
      now
    )

    expect(draft.blockers).toEqual([
      "Apply the migration needs input or approval.",
    ])
    expect(draft.nextActions[0]).toBe("Answer 1 waiting session")
  })

  test("counts failures as a risky pattern", () => {
    const draft = buildWeeklyReview(
      {
        reviews: [],
        sessions: [
          { createdAt: today, endedAt: today, prompt: null, status: "failed" },
          { createdAt: today, endedAt: today, prompt: null, status: "failed" },
        ],
      },
      now
    )

    expect(draft.riskyPatterns).toEqual([
      "2 sessions failed; check the command and the environment.",
    ])
    expect(draft.nextActions).toEqual(["Retry 2 failed sessions"])
  })

  test("keeps rejected reasons from the week only", () => {
    const draft = buildWeeklyReview(
      {
        reviews: [
          {
            action: "rejected",
            createdAt: today,
            reason: "Wrong environment.",
          },
          { action: "rejected", createdAt: lastWeek, reason: "Too old." },
          { action: "approved", createdAt: today, reason: null },
          { action: "pending", createdAt: today, reason: null },
        ],
        sessions: [],
      },
      now
    )

    expect(draft.rejectedChanges).toEqual(["Wrong environment."])
    expect(draft.summary).toBe(
      "0 done, 0 waiting, 0 failed, 1 rejected this week."
    )
    expect(draft.nextActions).toEqual(["Review 1 change"])
  })

  test("orders actions by how blocking they are, at most three", () => {
    const draft = buildWeeklyReview(
      {
        reviews: [{ action: "pending", createdAt: today, reason: null }],
        sessions: [
          {
            createdAt: today,
            endedAt: null,
            prompt: "Waiting work",
            status: "waiting",
          },
          { createdAt: today, endedAt: today, prompt: null, status: "failed" },
          { createdAt: today, endedAt: today, prompt: null, status: "ready" },
        ],
      },
      now
    )

    expect(draft.nextActions).toEqual([
      "Answer 1 waiting session",
      "Review 1 change",
      "Retry 1 failed session",
    ])
  })
})
