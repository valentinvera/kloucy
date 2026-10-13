const LOCAL_FACTS = [
  "Detection resolves executables on your PATH. It never runs them.",
  "Commands run with sh -c inside the workspace path you selected.",
  "Stop terminates the agent process group, so child processes stop too.",
  "Sessions, logs, reviews, and decisions stay in your own database.",
]

export function LandingLocal() {
  return (
    <div className="grid grid-cols-1 gap-8 lg:grid-cols-2 lg:gap-12">
      <div className="flex flex-col gap-2 border border-border/60 bg-card p-4 font-mono text-demo-metadata">
        <span className="text-muted-foreground">~/projects/acme-api</span>
        <span className="text-muted-foreground">$ bun run dev:api</span>
        <span>Kloucy API listening on 0.0.0.0:4040</span>
        <span className="text-muted-foreground">$ codex exec --full-auto</span>
        <span className="text-info-foreground">Running · 00:41</span>
      </div>
      <ul className="flex flex-col divide-y divide-border/60">
        {LOCAL_FACTS.map((fact) => (
          <li
            className="py-3 text-body text-muted-foreground first:pt-0"
            key={fact}
          >
            {fact}
          </li>
        ))}
      </ul>
    </div>
  )
}
