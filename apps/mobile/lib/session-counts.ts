export interface SessionCounts {
  blocked: number
  done: number
  failed: number
  ready: number
  running: number
  total: number
}

const STATUS_GROUPS: Record<string, keyof SessionCounts> = {
  done: "done",
  failed: "failed",
  ready: "ready",
  running: "running",
  waiting: "blocked",
}

export function sessionCounts(sessions: { status: string }[]): SessionCounts {
  const counts: SessionCounts = {
    blocked: 0,
    done: 0,
    failed: 0,
    ready: 0,
    running: 0,
    total: sessions.length,
  }

  for (const session of sessions) {
    const group = STATUS_GROUPS[session.status]

    if (group) {
      counts[group] += 1
    }
  }

  return counts
}

/** One line describing the workspace state, blocked work first. */
export function summarizeCounts(counts: SessionCounts): string {
  if (counts.total === 0) {
    return "No sessions yet."
  }

  const parts = [
    counts.blocked > 0 ? `${counts.blocked} waiting` : null,
    counts.running > 0 ? `${counts.running} running` : null,
    counts.failed > 0 ? `${counts.failed} failed` : null,
    counts.ready > 0 ? `${counts.ready} ready` : null,
    counts.done > 0 ? `${counts.done} done` : null,
  ].filter((part): part is string => part !== null)

  return parts.length > 0 ? parts.join(" · ") : "Nothing running."
}
