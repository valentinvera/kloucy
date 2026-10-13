export interface NextActionSession {
  id: string
  prompt: string | null
  status: string
}

export interface NextActionReview {
  action: string
  id: string
}

export interface NextActionInput {
  reviews: NextActionReview[]
  sessions: NextActionSession[]
}

export interface NextAction {
  detail: string
  title: string
}

function plural(count: number, singular: string): string {
  return count === 1 ? singular : `${singular}s`
}

function sessionLabel(session: NextActionSession): string {
  return session.prompt ?? "Untitled session"
}

/**
 * At most three actions, most blocking first: sessions that need input, changes
 * that need approval, failures, work ready to start, then what is running.
 */
export function nextActions(input: NextActionInput): NextAction[] {
  const byStatus = (status: string): NextActionSession[] =>
    input.sessions.filter((session) => session.status === status)

  const waiting = byStatus("waiting")
  const failed = byStatus("failed")
  const ready = byStatus("ready")
  const running = byStatus("running")
  const pendingReviews = input.reviews.filter(
    (review) => review.action === "pending"
  )

  const actions: NextAction[] = []

  if (waiting.length > 0) {
    actions.push({
      detail: `${sessionLabel(waiting[0])} needs input or approval.`,
      title: `Answer ${waiting.length} waiting ${plural(waiting.length, "session")}`,
    })
  }

  if (pendingReviews.length > 0) {
    actions.push({
      detail: "Approve or reject what the agent wrote.",
      title: `Review ${pendingReviews.length} ${plural(pendingReviews.length, "change")}`,
    })
  }

  if (failed.length > 0) {
    actions.push({
      detail: `${sessionLabel(failed[0])} ended with an error.`,
      title: `Retry ${failed.length} failed ${plural(failed.length, "session")}`,
    })
  }

  if (ready.length > 0) {
    actions.push({
      detail: `${sessionLabel(ready[0])} is waiting to run.`,
      title: `Start ${ready.length} ready ${plural(ready.length, "session")}`,
    })
  }

  if (running.length > 0) {
    actions.push({
      detail: "Output is streaming to the session log.",
      title: `${running.length} ${plural(running.length, "session")} running`,
    })
  }

  if (actions.length === 0) {
    actions.push({
      detail: "Start a task on desktop to run an agent.",
      title: "Nothing needs you",
    })
  }

  return actions.slice(0, 3)
}
