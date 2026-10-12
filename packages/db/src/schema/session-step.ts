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

export const sessionLaneEnum = pgEnum("session_lane", [
  "now",
  "next",
  "later",
  "not_doing",
])

export const sessionStep = pgTable(
  "session_step",
  {
    createdAt: timestamp("created_at").defaultNow().notNull(),
    id: text("id").primaryKey(),
    lane: sessionLaneEnum("lane").default("now").notNull(),
    reason: text("reason"),
    sessionId: text("session_id")
      .notNull()
      .references(() => agentSession.id, { onDelete: "cascade" }),
    sortOrder: integer("sort_order").default(0).notNull(),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => /* @__PURE__ */ new Date())
      .notNull(),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [index("session_step_sessionId_idx").on(table.sessionId)]
)

export const sessionStepRelations = relations(sessionStep, ({ one }) => ({
  session: one(agentSession, {
    fields: [sessionStep.sessionId],
    references: [agentSession.id],
  }),
  workspace: one(workspace, {
    fields: [sessionStep.workspaceId],
    references: [workspace.id],
  }),
}))
