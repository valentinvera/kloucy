import { env } from "@kloucy/env/api"
import { neon } from "@neondatabase/serverless"
import { drizzle } from "drizzle-orm/neon-http"

import { agent, agentRelations } from "./schema/agent"
import {
  account,
  accountRelations,
  session,
  sessionRelations,
  user,
  userRelations,
  verification,
} from "./schema/auth"
import { workspace, workspaceRelations } from "./schema/workspace"

const schema = {
  account,
  accountRelations,
  agent,
  agentRelations,
  session,
  sessionRelations,
  user,
  userRelations,
  verification,
  workspace,
  workspaceRelations,
}

export function createDb() {
  const sql = neon(env.DATABASE_URL)
  return drizzle(sql, { schema })
}

export const db = createDb()
