import { relations } from "drizzle-orm"
import {
  boolean,
  index,
  pgEnum,
  pgTable,
  text,
  timestamp,
} from "drizzle-orm/pg-core"
import { user } from "./auth"

export const workspaceStageEnum = pgEnum("workspace_stage", [
  "setup",
  "ready",
  "active",
  "paused",
  "archived",
])

export const workspaceSafetyModeEnum = pgEnum("workspace_safety_mode", [
  "open",
  "guarded",
])

export const workspace = pgTable(
  "workspace",
  {
    archivedAt: timestamp("archived_at"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    defaultAgentId: text("default_agent_id"),
    description: text("description"),
    id: text("id").primaryKey(),
    ignoredPaths: text("ignored_paths").array(),
    localPath: text("local_path").notNull(),
    mobilePairingEnabled: boolean("mobile_pairing_enabled")
      .default(false)
      .notNull(),
    name: text("name").notNull(),
    ownerId: text("owner_id")
      .notNull()
      .references(() => user.id, { onDelete: "cascade" }),
    packageManager: text("package_manager"),
    projectType: text("project_type"),
    runtime: text("runtime"),
    safeCommands: text("safe_commands").array(),
    safetyMode: workspaceSafetyModeEnum("safety_mode")
      .default("open")
      .notNull(),
    stage: workspaceStageEnum("stage").default("setup").notNull(),
    updatedAt: timestamp("updated_at")
      .defaultNow()
      .$onUpdate(() => /* @__PURE__ */ new Date())
      .notNull(),
  },
  (table) => [index("workspace_ownerId_idx").on(table.ownerId)]
)

export const workspaceRelations = relations(workspace, ({ one }) => ({
  owner: one(user, {
    fields: [workspace.ownerId],
    references: [user.id],
  }),
}))
