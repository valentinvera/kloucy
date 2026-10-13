const STATUS_LABEL: Record<string, string> = {
  done: "Done",
  failed: "Failed",
  needs_setup: "Needs setup",
  ready: "Ready",
  running: "Running",
  stopped: "Stopped",
  waiting: "Waiting",
}

const STATUS_CLASS: Record<string, string> = {
  done: "bg-success text-success-foreground",
  failed: "bg-destructive text-destructive-foreground",
  needs_setup: "bg-secondary text-secondary-foreground",
  ready: "bg-secondary text-secondary-foreground",
  running: "bg-info text-info-foreground",
  stopped: "bg-muted text-muted-foreground",
  waiting: "bg-warning text-warning-foreground",
}

export function sessionStatusLabel(status: string): string {
  return STATUS_LABEL[status] ?? status
}

export function sessionStatusClass(status: string): string {
  return STATUS_CLASS[status] ?? ""
}
