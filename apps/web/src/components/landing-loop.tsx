import { Separator } from "@kloucy/ui/components/separator"
import type { ReactNode } from "react"

interface LoopStepProps {
  children: ReactNode
  detail: string
  index: string
  title: string
}

function LoopStep({ children, detail, index, title }: LoopStepProps) {
  return (
    <div className="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)] lg:gap-12">
      <div className="flex min-w-0 flex-col gap-2">
        <span className="font-medium text-caption text-muted-foreground">
          {index}
        </span>
        <h3 className="font-medium text-medium">{title}</h3>
        <p className="max-w-84 text-body text-muted-foreground">{detail}</p>
      </div>
      <div className="min-w-0">{children}</div>
    </div>
  )
}

const CHANGED_FILES = [
  { additions: "+12", deletions: "−4", path: "apps/api/src/main.ts" },
  { additions: "+38", deletions: "", path: "packages/db/drizzle/0002.sql" },
  { additions: "+64", deletions: "−3", path: "apps/web/src/routes/index.tsx" },
]

export function LandingLoop() {
  return (
    <div className="flex flex-col gap-8">
      <LoopStep
        detail="A task holds the intent. It waits until you run it."
        index="01"
        title="Capture the work"
      >
        <div className="flex flex-col gap-2 border border-border/60 bg-card p-4">
          <span className="text-demo-control">Rotate the staging token</span>
          <p className="text-demo-body text-muted-foreground">
            Rotate the token in the API, then update the mobile environment.
          </p>
          <span className="text-demo-metadata text-muted-foreground">
            Captured from mobile · 12 minutes ago
          </span>
        </div>
      </LoopStep>

      <Separator />

      <LoopStep
        detail="The agent starts in the workspace path and streams everything it does."
        index="02"
        title="Run it in your project"
      >
        <div className="flex flex-col gap-2 border border-border/60 bg-card p-4 font-mono text-demo-metadata">
          <span className="text-muted-foreground">
            $ codex exec --full-auto
          </span>
          <span>reading apps/api/src/main.ts</span>
          <span>updating packages/env/src/api.ts</span>
          <span className="text-info-foreground">Running · 00:41</span>
        </div>
      </LoopStep>

      <Separator />

      <LoopStep
        detail="Changed files become a review you approve or reject."
        index="03"
        title="Review what changed"
      >
        <div className="flex flex-col gap-3 border border-border/60 bg-card p-4">
          <span className="text-demo-metadata text-muted-foreground">
            3 files changed · 2 minutes ago
          </span>
          <div className="flex flex-col gap-1 font-mono text-demo-metadata">
            {CHANGED_FILES.map((file) => (
              <div
                className="flex items-center justify-between gap-4"
                key={file.path}
              >
                <span className="truncate">{file.path}</span>
                <span className="shrink-0">
                  <span className="text-success-foreground">
                    {file.additions}
                  </span>
                  {file.deletions ? (
                    <span className="ml-2 text-destructive-foreground">
                      {file.deletions}
                    </span>
                  ) : null}
                </span>
              </div>
            ))}
          </div>
          <span className="text-demo-metadata text-warning-foreground">
            Waiting for your approval.
          </span>
        </div>
      </LoopStep>
    </div>
  )
}
