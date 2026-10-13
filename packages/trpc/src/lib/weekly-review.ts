export interface WeeklyActivitySession {
  createdAt: Date
  endedAt: Date | null
  prompt: string | null
  status: string
}

export interface WeeklyActivityReview {
  action: string
  createdAt: Date
  reason: string | null
}

export interface WeeklyActivity {
  reviews: WeeklyActivityReview[]
  sessions: WeeklyActivitySession[]
}

export interface WeeklyReviewDraft {
  blockers: string[]
  nextActions: string[]
  rejectedChanges: string[]
  riskyPatterns: string[]
  shipped: string[]
  summary: string
}

export const WEEK_MS = 7 * 24 * 60 * 60 * 1000

function plural(count: number, singular: string): string {
  return count === 1 ? singular : `${singular}s`
}

function sessionLabel(session: WeeklyActivitySession): string {
  return session.prompt ?? "Untitled session"
}

function nextActions(input: {
  failed: WeeklyActivitySession[]
  pendingReviews: number
  shipped: number
  waiting: WeeklyActivitySession[]
}): string[] {
  const actions: string[] = []

  if (input.waiting.length > 0) {
    actions.push(
      `Answer ${input.waiting.length} waiting ${plural(input.waiting.length, "session")}`
    )
  }

  if (input.pendingReviews > 0) {
    actions.push(
      `Review ${input.pendingReviews} ${plural(input.pendingReviews, "change")}`
    )
  }

  if (input.failed.length > 0) {
    actions.push(
      `Retry ${input.failed.length} failed ${plural(input.failed.length, "session")}`
    )
  }

  if (actions.length === 0) {
    actions.push(
      input.shipped > 0 ? "Nothing needs you" : "Start a task from the inbox"
    )
  }

  return actions.slice(0, 3)
}

/**
 * Derive a weekly review from stored agent activity.
 *
 * Sessions count for the week when they were created or ended inside it.
 * Waiting sessions are current blockers, so they are reported regardless of
 * when they started.
 */
export function buildWeeklyReview(
  activity: WeeklyActivity,
  now: Date
): WeeklyReviewDraft {
  const since = now.getTime() - WEEK_MS
  const inWeek = (value: Date | null): boolean =>
    value !== null && value.getTime() >= since

  const thisWeek = activity.sessions.filter(
    (session) => inWeek(session.createdAt) || inWeek(session.endedAt)
  )

  const shipped = thisWeek
    .filter((session) => session.status === "done")
    .map((session) => sessionLabel(session))
  const failed = thisWeek.filter((session) => session.status === "failed")
  const waiting = activity.sessions.filter(
    (session) => session.status === "waiting"
  )
  const pendingReviews = activity.reviews.filter(
    (review) => review.action === "pending"
  )
  const rejected = activity.reviews.filter(
    (review) => review.action === "rejected" && inWeek(review.createdAt)
  )

  const blockers = waiting.map(
    (session) => `${sessionLabel(session)} needs input or approval.`
  )
  const rejectedChanges = rejected.map(
    (review) => review.reason ?? "Changed files"
  )
  const riskyPatterns =
    failed.length > 0
      ? [
          `${failed.length} ${plural(failed.length, "session")} failed; check the command and the environment.`,
        ]
      : []

  return {
    blockers,
    nextActions: nextActions({
      failed,
      pendingReviews: pendingReviews.length,
      shipped: shipped.length,
      waiting,
    }),
    rejectedChanges,
    riskyPatterns,
    shipped,
    summary: `${shipped.length} done, ${waiting.length} waiting, ${failed.length} failed, ${rejected.length} rejected this week.`,
  }
}
