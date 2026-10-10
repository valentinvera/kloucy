import { relations } from "drizzle-orm"
import { index, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { user } from "./auth"
import { workspace } from "./workspace"

export const agent = pgTable(
  "agent",
  {
    capabilities: text("capabilities").array(),
    commandTemplate: text("command_template"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    defaultWorkspaceId: text("default_workspace_id").references(
      () => workspace.id,
      { onDelete: "set null" }
    ),
    executablePath: text("executable_path"),
    id: text("id").primaryKey(),
    lastUsedAt: timestamp("last_used_at"),
    name: text("name").notNull(),
    ownerId: text("owner_id")
      .notNull()
      .references(() => user.id, { onDelete: "cascade" }),
    permissionMode: text("permission_mode"),
    provider: text("provider").notNull(),
    status: text("status").default("needs_setup").notNull(),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => /* @__PURE__ */ new Date())
      .notNull(),
  },
  (table) => [index("agent_ownerId_idx").on(table.ownerId)]
)

export const agentRelations = relations(agent, ({ one }) => ({
  defaultWorkspace: one(workspace, {
    fields: [agent.defaultWorkspaceId],
    references: [workspace.id],
  }),
  owner: one(user, {
    fields: [agent.ownerId],
    references: [user.id],
  }),
}))
