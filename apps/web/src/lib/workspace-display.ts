const STAGE_LABEL: Record<string, string> = {
  active: "Active",
  archived: "Archived",
  paused: "Paused",
  ready: "Ready",
  setup: "Setup",
}

const STAGE_CLASS: Record<string, string> = {
  active: "border-transparent bg-success text-success-foreground",
  archived: "border-transparent bg-muted text-muted-foreground",
  paused: "border-transparent bg-warning text-warning-foreground",
  ready: "border-transparent bg-info text-info-foreground",
  setup: "border-transparent bg-secondary text-secondary-foreground",
}

const dateFormatter = new Intl.DateTimeFormat("en", { dateStyle: "medium" })

export function stageLabel(stage: string): string {
  return STAGE_LABEL[stage] ?? stage
}

export function stageClass(stage: string): string {
  return STAGE_CLASS[stage] ?? ""
}

export function formatDate(value: Date | string): string {
  return dateFormatter.format(new Date(value))
}
