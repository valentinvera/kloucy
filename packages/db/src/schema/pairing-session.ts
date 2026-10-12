import { relations } from "drizzle-orm"
import { index, pgTable, text, timestamp } from "drizzle-orm/pg-core"
import { user } from "./auth"

export const pairingSession = pgTable(
  "pairing_session",
  {
    claimedAt: timestamp("claimed_at"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    deviceName: text("device_name"),
    expiresAt: timestamp("expires_at").notNull(),
    id: text("id").primaryKey(),
    ownerId: text("owner_id")
      .notNull()
      .references(() => user.id, { onDelete: "cascade" }),
    status: text("status").default("pending").notNull(),
    token: text("token").notNull().unique(),
  },
  (table) => [index("pairing_session_ownerId_idx").on(table.ownerId)]
)

export const pairingSessionRelations = relations(pairingSession, ({ one }) => ({
  owner: one(user, {
    fields: [pairingSession.ownerId],
    references: [user.id],
  }),
}))
