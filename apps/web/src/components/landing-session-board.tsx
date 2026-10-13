import { Badge } from "@kloucy/ui/components/badge"
import { cn } from "@kloucy/ui/lib/utils"
import { sessionStatusClass, sessionStatusLabel } from "@/lib/session-status"

interface DemoSession {
  agent: string
  detail: string
  title: string
}

interface DemoLane {
  sessions: DemoSession[]
  status: string
}

const LANES: DemoLane[] = [
  {
    sessions: [
      {
        agent: "Codex",
        detail: "captured from mobile",
        title: "Rotate the staging token",
      },
    ],
    status: "ready",
  },
  {
    sessions: [
      {
        agent: "Claude Code",
        detail: "running for 00:41",
        title: "Add the readiness snapshot",
      },
    ],
    status: "running",
  },
  {
    sessions: [
      {
        agent: "Codex",
        detail: "waiting for approval to run migrations",
        title: "Apply the drizzle migration",
      },
    ],
    status: "waiting",
  },
  {
    sessions: [
      {
        agent: "Aider",
        detail: "3 files changed",
        title: "Tighten the review row",
      },
    ],
    status: "done",
  },
]

export function LandingSessionBoard() {
  return (
    <div className="border border-border/60 bg-card">
      <div className="flex items-center justify-between gap-4 border-border/60 border-b px-4 py-3">
        <div className="flex min-w-0 items-baseline gap-2">
          <span className="font-medium text-medium">acme-api</span>
          <span className="truncate text-caption text-muted-foreground">
            ~/projects/acme-api
          </span>
        </div>
        <span className="shrink-0 text-caption text-muted-foreground">
          4 sessions
        </span>
      </div>
      <div className="flex overflow-x-auto">
        {LANES.map((lane) => (
          <div
            className="flex min-w-56 flex-1 flex-col gap-3 border-border/60 border-r p-4 last:border-r-0"
            key={lane.status}
          >
            <div className="flex items-center justify-between gap-2">
              <Badge
                className={cn(
                  "border-transparent",
                  sessionStatusClass(lane.status)
                )}
              >
                {sessionStatusLabel(lane.status)}
              </Badge>
              <span className="text-caption text-muted-foreground">
                {lane.sessions.length}
              </span>
            </div>
            {lane.sessions.map((session) => (
              <div className="flex flex-col gap-1" key={session.title}>
                <span className="text-demo-control">{session.title}</span>
                <span
                  className={cn(
                    "text-demo-metadata",
                    lane.status === "waiting"
                      ? "text-warning-foreground"
                      : "text-muted-foreground"
                  )}
                >
                  {session.agent} · {session.detail}
                </span>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  )
}
