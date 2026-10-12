import { relations } from "drizzle-orm"
import {
  index,
  integer,
  pgEnum,
  pgTable,
  text,
  timestamp,
} from "drizzle-orm/pg-core"
import { agentSession } from "./agent-session"
import { workspace } from "./workspace"

export const readinessStatusEnum = pgEnum("readiness_status", [
  "not_ready",
  "almost_ready",
  "ready_to_run",
])

export const readinessSnapshot = pgTable(
  "readiness_snapshot",
  {
    blockers: text("blockers").array(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    id: text("id").primaryKey(),
    notRequiredYet: text("not_required_yet").array(),
    score: integer("score").notNull(),
    sessionId: text("session_id").references(() => agentSession.id, {
      onDelete: "set null",
    }),
    status: readinessStatusEnum("status").notNull(),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [index("readiness_snapshot_workspaceId_idx").on(table.workspaceId)]
)

export const readinessSnapshotRelations = relations(
  readinessSnapshot,
  ({ one }) => ({
    session: one(agentSession, {
      fields: [readinessSnapshot.sessionId],
      references: [agentSession.id],
    }),
    workspace: one(workspace, {
      fields: [readinessSnapshot.workspaceId],
      references: [workspace.id],
    }),
  })
)
