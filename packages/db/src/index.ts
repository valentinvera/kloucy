import { env } from "@kloucy/env/api"
import { neon } from "@neondatabase/serverless"
import { drizzle } from "drizzle-orm/neon-http"

import { agent, agentRelations } from "./schema/agent"
import { agentSession, agentSessionRelations } from "./schema/agent-session"
import { agentTask, agentTaskRelations } from "./schema/agent-task"
import {
  account,
  accountRelations,
  session,
  sessionRelations,
  user,
  userRelations,
  verification,
} from "./schema/auth"
import { decision, decisionRelations } from "./schema/decision"
import { review, reviewRelations } from "./schema/review"
import { workspace, workspaceRelations } from "./schema/workspace"

const schema = {
  account,
  accountRelations,
  agent,
  agentRelations,
  agentSession,
  agentSessionRelations,
  agentTask,
  agentTaskRelations,
  decision,
  decisionRelations,
  review,
  reviewRelations,
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
