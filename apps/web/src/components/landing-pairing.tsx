const PAIRING_FACTS = [
  "Check what is running and what is waiting.",
  "Send short follow-up instructions.",
  "Approve or reject changes.",
]

export function LandingPairing() {
  return (
    <div className="flex flex-col gap-6 border border-border/60 bg-card p-6 lg:flex-row lg:items-center lg:justify-between lg:gap-12">
      <div className="flex min-w-0 flex-col gap-2">
        <span className="font-medium text-demo-title">Mobile paired.</span>
        <span className="truncate font-mono text-demo-metadata text-muted-foreground">
          kloucy://pair?token=8f1c4c1e-7d3a-4a55-9a2d
        </span>
        <span className="text-demo-metadata text-success-foreground">
          claimed · iPhone · 2 minutes ago
        </span>
      </div>
      <ul className="flex flex-col gap-3 lg:gap-2">
        {PAIRING_FACTS.map((fact) => (
          <li className="text-body text-muted-foreground" key={fact}>
            {fact}
          </li>
        ))}
      </ul>
    </div>
  )
}
