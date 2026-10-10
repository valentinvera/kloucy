import type { ReactNode } from "react"

interface AuthShellProps {
  children: ReactNode
  description: string
  footer?: ReactNode
  title: string
}

export function AuthShell({
  children,
  description,
  footer,
  title,
}: AuthShellProps) {
  return (
    <main className="grid min-h-svh grid-cols-1 lg:grid-cols-2">
      <section className="hidden flex-col justify-between border-border/60 border-r px-10 py-10 lg:flex">
        <span className="font-logo text-section">Kloucy</span>
        <p className="max-w-xs text-medium text-muted-foreground">
          Run local agents, see what changed, and stay in control.
        </p>
      </section>
      <section className="flex flex-col justify-center px-6 py-12 sm:px-10">
        <div className="mx-auto flex w-full max-w-sm flex-col">
          <h1 className="font-medium text-section">{title}</h1>
          <p className="mt-2 text-medium text-muted-foreground">
            {description}
          </p>
          <div className="mt-8">{children}</div>
          {footer ? <div className="mt-6">{footer}</div> : null}
        </div>
      </section>
    </main>
  )
}
