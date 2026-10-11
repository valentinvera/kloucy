import { relations } from "drizzle-orm"
import { index, pgEnum, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { agent } from "./agent"
import { agentTask } from "./agent-task"
import { workspace } from "./workspace"

export const agentSessionStatusEnum = pgEnum("agent_session_status", [
  "needs_setup",
  "ready",
  "running",
  "waiting",
  "done",
  "failed",
  "stopped",
])

export const agentSession = pgTable(
  "agent_session",
  {
    agentId: text("agent_id").references(() => agent.id, {
      onDelete: "set null",
    }),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    endedAt: timestamp("ended_at"),
    id: text("id").primaryKey(),
    prompt: text("prompt"),
    startedAt: timestamp("started_at"),
    status: agentSessionStatusEnum("status").default("needs_setup").notNull(),
    taskId: text("task_id").references(() => agentTask.id, {
      onDelete: "set null",
    }),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => /* @__PURE__ */ new Date())
      .notNull(),
    workspaceId: text("workspace_id")
      .notNull()
      .references(() => workspace.id, { onDelete: "cascade" }),
  },
  (table) => [index("agent_session_workspaceId_idx").on(table.workspaceId)]
)

export const agentSessionRelations = relations(agentSession, ({ one }) => ({
  agent: one(agent, {
    fields: [agentSession.agentId],
    references: [agent.id],
  }),
  task: one(agentTask, {
    fields: [agentSession.taskId],
    references: [agentTask.id],
  }),
  workspace: one(workspace, {
    fields: [agentSession.workspaceId],
    references: [workspace.id],
  }),
}))
