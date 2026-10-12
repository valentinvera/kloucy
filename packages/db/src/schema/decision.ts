import { relations } from "drizzle-orm"
import { index, pgEnum, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { agentSession } from "./agent-session"
import { workspace } from "./workspace"

export const decisionTypeEnum = pgEnum("decision_type", [
  "scope",
  "permission",
  "run",
  "review",
  "reject",
  "revisit",
  "note",
])

export const decision = pgTable(
  "decision",
  {
    createdAt: timestamp("created_at").defaultNow().notNull(),
    fromValue: text("from_value"),
    id: text("id").primaryKey(),
    reason: text("reason"),
    revisitWhen: text("revisit_when"),
    sessionId: text("session_id").references(() => agentSession.id, {
      onDelete: "set null",
    }),
    summary: text("summary"),
    title: text("title").notNull(),
    toValue: text("to_value"),
    type: decisionTypeEnum("type").default("note").notNull(),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [index("decision_workspaceId_idx").on(table.workspaceId)]
)

export const decisionRelations = relations(decision, ({ one }) => ({
  session: one(agentSession, {
    fields: [decision.sessionId],
    references: [agentSession.id],
  }),
  workspace: one(workspace, {
    fields: [decision.workspaceId],
    references: [workspace.id],
  }),
}))
