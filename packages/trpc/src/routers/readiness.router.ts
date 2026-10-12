import { db } from "@kloucy/db"
import { readinessSnapshot } from "@kloucy/db/schema/readiness-snapshot"
import { TRPCError } from "@trpc/server"
import { and, eq } from "drizzle-orm"
import {
  Ctx,
  Input,
  Mutation,
  Query,
  Router,
  UseMiddlewares,
} from "nestjs-trpc"
import { z } from "zod"
import type { Context } from "../context"
import { ProtectedMiddleware } from "../middleware/protected"

const readinessStatusSchema = z.enum([
  "not_ready",
  "almost_ready",
  "ready_to_run",
])

const readinessSnapshotSchema = z.object({
  blockers: z.array(z.string()).nullable(),
  createdAt: z.date(),
  id: z.string(),
  notRequiredYet: z.array(z.string()).nullable(),
  score: z.number(),
  sessionId: z.string().nullable(),
  status: readinessStatusSchema,
  workspaceId: z.string(),
})

const readinessCalculateInputSchema = z.object({
  sessionId: z.string().min(1).optional(),
  workspaceId: z.string().min(1),
})

const readinessWorkspaceInputSchema = z.object({
  workspaceId: z.string().min(1),
})

function requireUserId(ctx: Context): string {
  if (!ctx.session?.user) {
    throw new TRPCError({
      code: "UNAUTHORIZED",
      message: "Authentication required",
    })
  }

  return ctx.session.user.id
}

async function assertWorkspaceOwned(
  workspaceId: string,
  ownerId: string
): Promise<void> {
  const found = await db.query.workspace.findFirst({
    where: (table) =>
      and(eq(table.id, workspaceId), eq(table.ownerId, ownerId)),
  })

  if (!found) {
    throw new TRPCError({
      code: "BAD_REQUEST",
      message: "Workspace not found",
    })
  }
}

function scoreStatus(score: number): z.infer<typeof readinessStatusSchema> {
  if (score >= 80) {
    return "ready_to_run"
  }
  if (score >= 50) {
    return "almost_ready"
  }
  return "not_ready"
}

@Router({ alias: "readiness" })
export class ReadinessRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({
    input: readinessCalculateInputSchema,
    output: readinessSnapshotSchema,
  })
  async calculate(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof readinessCalculateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    const workspace = await db.query.workspace.findFirst({
      where: (table) =>
        and(eq(table.id, input.workspaceId), eq(table.ownerId, ownerId)),
    })

    if (!workspace) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "Workspace not found",
      })
    }

    const agents = await db.query.agent.findMany({
      where: (table) => eq(table.ownerId, ownerId),
    })
    const tasks = await db.query.agentTask.findMany({
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })
    const reviews = await db.query.review.findMany({
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })

    const readyTaskMissingPrompt = tasks.some(
      (task) => task.status === "ready" && !task.prompt
    )

    const criteria = [
      {
        label: "Workspace has a local path",
        met: workspace.localPath.length > 0,
      },
      {
        label: "Workspace has a default agent",
        met: workspace.defaultAgentId !== null,
      },
      {
        label: "Workspace has safe commands",
        met: (workspace.safeCommands?.length ?? 0) > 0,
      },
      { label: "At least one agent is configured", met: agents.length > 0 },
      { label: "At least one task is captured", met: tasks.length > 0 },
      {
        label: "No ready task is missing a prompt",
        met: !readyTaskMissingPrompt,
      },
      { label: "A review has been recorded", met: reviews.length > 0 },
    ]

    const met = criteria.filter((criterion) => criterion.met).length
    const score = Math.round((met / criteria.length) * 100)

    const [snapshot] = await db
      .insert(readinessSnapshot)
      .values({
        blockers: criteria
          .filter((criterion) => !criterion.met)
          .map((criterion) => criterion.label),
        id: crypto.randomUUID(),
        notRequiredYet: [
          "Mobile pairing",
          "Weekly review",
          "Team collaboration",
        ],
        score,
        sessionId: input.sessionId ?? null,
        status: scoreStatus(score),
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!snapshot) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to calculate readiness",
      })
    }

    return snapshot
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({
    input: readinessWorkspaceInputSchema,
    output: readinessSnapshotSchema.nullable(),
  })
  async latest(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof readinessWorkspaceInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    const found = await db.query.readinessSnapshot.findFirst({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })

    return found ?? null
  }
}
