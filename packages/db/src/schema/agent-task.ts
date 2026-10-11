import { relations } from "drizzle-orm"
import { index, pgEnum, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { agent } from "./agent"
import { workspace } from "./workspace"

export const agentTaskStatusEnum = pgEnum("agent_task_status", [
  "new",
  "needs_setup",
  "ready",
  "running",
  "later",
  "rejected",
  "converted",
])

export const agentTaskSourceEnum = pgEnum("agent_task_source", [
  "desktop",
  "mobile",
  "project_context",
  "user_note",
  "agent_suggestion",
  "other",
])

export const agentTask = pgTable(
  "agent_task",
  {
    assignedAgentId: text("assigned_agent_id").references(() => agent.id, {
      onDelete: "set null",
    }),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    createdSessionId: text("created_session_id"),
    id: text("id").primaryKey(),
    prompt: text("prompt"),
    source: agentTaskSourceEnum("source").default("desktop").notNull(),
    status: agentTaskStatusEnum("status").default("new").notNull(),
    title: text("title").notNull(),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => /* @__PURE__ */ new Date())
      .notNull(),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [index("agent_task_workspaceId_idx").on(table.workspaceId)]
)

export const agentTaskRelations = relations(agentTask, ({ one }) => ({
  assignedAgent: one(agent, {
    fields: [agentTask.assignedAgentId],
    references: [agent.id],
  }),
  workspace: one(workspace, {
    fields: [agentTask.workspaceId],
    references: [workspace.id],
  }),
}))
