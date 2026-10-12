import { relations } from "drizzle-orm"
import { index, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { agentSession } from "./agent-session"
import { workspace } from "./workspace"

export const review = pgTable(
  "review",
  {
    action: text("action").default("pending").notNull(),
    artifactId: text("artifact_id"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    id: text("id").primaryKey(),
    reason: text("reason"),
    sessionId: text("session_id")
      .notNull()
      .references(() => agentSession.id, { onDelete: "cascade" }),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [
    index("review_workspaceId_idx").on(table.workspaceId),
    index("review_sessionId_idx").on(table.sessionId),
  ]
)

export const reviewRelations = relations(review, ({ one }) => ({
  session: one(agentSession, {
    fields: [review.sessionId],
    references: [agentSession.id],
  }),
  workspace: one(workspace, {
    fields: [review.workspaceId],
    references: [workspace.id],
  }),
}))
