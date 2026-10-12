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
import {
  pairingSession,
  pairingSessionRelations,
} from "./schema/pairing-session"
import {
  readinessSnapshot,
  readinessSnapshotRelations,
} from "./schema/readiness-snapshot"
import { review, reviewRelations } from "./schema/review"
import { sessionStep, sessionStepRelations } from "./schema/session-step"
import { weeklyReview, weeklyReviewRelations } from "./schema/weekly-review"
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
  pairingSession,
  pairingSessionRelations,
  readinessSnapshot,
  readinessSnapshotRelations,
  review,
  reviewRelations,
  session,
  sessionRelations,
  sessionStep,
  sessionStepRelations,
  user,
  userRelations,
  verification,
  weeklyReview,
  weeklyReviewRelations,
  workspace,
  workspaceRelations,
}

export function createDb() {
  const sql = neon(env.DATABASE_URL)
  return drizzle(sql, { schema })
}

export const db = createDb()
