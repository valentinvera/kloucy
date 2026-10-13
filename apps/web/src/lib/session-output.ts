export interface SessionLogLine {
  id: string
  message: string
  stream: string
}

export interface SessionOutput {
  logs: SessionLogLine[]
  status: string
}

/**
 * Append freshly streamed lines, ignoring any the app already holds so a
 * repeated or overlapping page cannot duplicate output.
 */
export function nextLogs(
  previous: SessionLogLine[],
  incoming: SessionLogLine[]
): SessionLogLine[] {
  const seen = new Set(previous.map((log) => log.id))

  return [...previous, ...incoming.filter((log) => !seen.has(log.id))]
}
