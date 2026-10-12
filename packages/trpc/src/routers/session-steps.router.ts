import { db } from "@kloucy/db"
import { sessionStep } from "@kloucy/db/schema/session-step"
import { TRPCError } from "@trpc/server"
import { eq } from "drizzle-orm"
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

const sessionLaneSchema = z.enum(["now", "next", "later", "not_doing"])

const sessionStepSchema = z.object({
  createdAt: z.date(),
  id: z.string(),
  lane: sessionLaneSchema,
  reason: z.string().nullable(),
  sessionId: z.string(),
  sortOrder: z.number(),
  updatedAt: z.date(),
  workspaceId: z.string(),
})

const sessionStepIdInputSchema = z.object({
  id: z.string().min(1),
})

const sessionStepListInputSchema = z.object({
  sessionId: z.string().min(1),
})

const sessionStepCreateInputSchema = z.object({
  lane: sessionLaneSchema.optional(),
  reason: z.string().min(1).optional(),
  sessionId: z.string().min(1),
  sortOrder: z.number().int().optional(),
  workspaceId: z.string().min(1),
})

const sessionStepUpdateInputSchema = z.object({
  id: z.string().min(1),
  lane: sessionLaneSchema.optional(),
  reason: z.string().min(1).nullable().optional(),
  sortOrder: z.number().int().optional(),
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

function notFound(): TRPCError {
  return new TRPCError({ code: "NOT_FOUND", message: "Session step not found" })
}

async function assertSessionOwned(
  sessionId: string,
  ownerId: string
): Promise<void> {
  const found = await db.query.agentSession.findFirst({
    where: (table) => eq(table.id, sessionId),
    with: { workspace: true },
  })

  if (!found || found.workspace.ownerId !== ownerId) {
    throw new TRPCError({ code: "BAD_REQUEST", message: "Session not found" })
  }
}

async function assertStepOwned(stepId: string, ownerId: string): Promise<void> {
  const found = await db.query.sessionStep.findFirst({
    where: (table) => eq(table.id, stepId),
    with: { workspace: true },
  })

  if (!found || found.workspace.ownerId !== ownerId) {
    throw notFound()
  }
}

@Router({ alias: "sessionSteps" })
export class SessionStepsRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({
    input: sessionStepListInputSchema,
    output: z.array(sessionStepSchema),
  })
  async list(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionStepListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.sessionId, ownerId)

    return await db.query.sessionStep.findMany({
      orderBy: (table, { asc }) => [asc(table.lane), asc(table.sortOrder)],
      where: (table) => eq(table.sessionId, input.sessionId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: sessionStepCreateInputSchema, output: sessionStepSchema })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionStepCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.sessionId, ownerId)

    const [created] = await db
      .insert(sessionStep)
      .values({
        id: crypto.randomUUID(),
        lane: input.lane ?? "now",
        reason: input.reason ?? null,
        sessionId: input.sessionId,
        sortOrder: input.sortOrder ?? 0,
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create session step",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: sessionStepUpdateInputSchema, output: sessionStepSchema })
  async update(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionStepUpdateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertStepOwned(input.id, ownerId)

    const changes: Partial<typeof sessionStep.$inferInsert> = {}

    if (input.lane !== undefined) {
      changes.lane = input.lane
    }
    if (input.sortOrder !== undefined) {
      changes.sortOrder = input.sortOrder
    }
    if (input.reason !== undefined) {
      changes.reason = input.reason
    }

    if (Object.keys(changes).length === 0) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "No fields to update",
      })
    }

    const [updated] = await db
      .update(sessionStep)
      .set(changes)
      .where(eq(sessionStep.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: sessionStepIdInputSchema, output: sessionStepSchema })
  async delete(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionStepIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertStepOwned(input.id, ownerId)

    const [deleted] = await db
      .delete(sessionStep)
      .where(eq(sessionStep.id, input.id))
      .returning()

    if (!deleted) {
      throw notFound()
    }

    return deleted
  }
}
