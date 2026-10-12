import { relations } from "drizzle-orm"
import { index, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { workspace } from "./workspace"

export const weeklyReview = pgTable(
  "weekly_review",
  {
    blockers: text("blockers").array(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    id: text("id").primaryKey(),
    nextActions: text("next_actions").array(),
    rejectedChanges: text("rejected_changes").array(),
    riskyPatterns: text("risky_patterns").array(),
    shipped: text("shipped").array(),
    summary: text("summary"),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [index("weekly_review_workspaceId_idx").on(table.workspaceId)]
)

export const weeklyReviewRelations = relations(weeklyReview, ({ one }) => ({
  workspace: one(workspace, {
    fields: [weeklyReview.workspaceId],
    references: [workspace.id],
  }),
}))
