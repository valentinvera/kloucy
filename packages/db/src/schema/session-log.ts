import { relations } from "drizzle-orm"
import { index, pgEnum, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { agentSession } from "./agent-session"

export const sessionLogStreamEnum = pgEnum("session_log_stream", [
  "stdout",
  "stderr",
  "system",
])

export const sessionLog = pgTable(
  "session_log",
  {
    createdAt: timestamp("created_at").defaultNow().notNull(),
    id: text("id").primaryKey(),
    message: text("message").notNull(),
    sessionId: text("session_id")
      .notNull()
      .references(() => agentSession.id, { onDelete: "cascade" }),
    stream: sessionLogStreamEnum("stream").default("stdout").notNull(),
  },
  (table) => [index("session_log_sessionId_idx").on(table.sessionId)]
)

export const sessionLogRelations = relations(sessionLog, ({ one }) => ({
  session: one(agentSession, {
    fields: [sessionLog.sessionId],
    references: [agentSession.id],
  }),
}))
