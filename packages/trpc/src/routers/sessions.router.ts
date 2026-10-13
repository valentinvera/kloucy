import { db } from "@kloucy/db"
import { agentSession } from "@kloucy/db/schema/agent-session"
import { sessionLog } from "@kloucy/db/schema/session-log"
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

const agentSessionStatusSchema = z.enum([
  "needs_setup",
  "ready",
  "running",
  "waiting",
  "done",
  "failed",
  "stopped",
])

const agentSessionSchema = z.object({
  agentId: z.string().nullable(),
  createdAt: z.date(),
  endedAt: z.date().nullable(),
  id: z.string(),
  prompt: z.string().nullable(),
  startedAt: z.date().nullable(),
  status: agentSessionStatusSchema,
  taskId: z.string().nullable(),
  updatedAt: z.date(),
  workspaceId: z.string(),
})

const agentSessionIdInputSchema = z.object({
  id: z.string().min(1),
})

const sessionInstructionInputSchema = z.object({
  id: z.string().min(1),
  instruction: z.string().min(1),
})

const agentSessionListInputSchema = z.object({
  workspaceId: z.string().min(1),
})

const agentSessionCreateInputSchema = z.object({
  agentId: z.string().min(1).optional(),
  prompt: z.string().min(1).optional(),
  status: agentSessionStatusSchema.optional(),
  taskId: z.string().min(1).optional(),
  workspaceId: z.string().min(1),
})

const sessionLogStreamSchema = z.enum(["stdout", "stderr", "system"])

const sessionLogSchema = z.object({
  createdAt: z.date(),
  id: z.string(),
  message: z.string(),
  sessionId: z.string(),
  stream: sessionLogStreamSchema,
})

const sessionLogListInputSchema = z.object({
  sessionId: z.string().min(1),
})

const sessionLogAppendInputSchema = z.object({
  message: z.string().min(1),
  sessionId: z.string().min(1),
  stream: sessionLogStreamSchema.optional(),
})

const sessionFinishInputSchema = z.object({
  id: z.string().min(1),
  status: z.enum(["done", "failed"]),
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
  return new TRPCError({ code: "NOT_FOUND", message: "Session not found" })
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

async function assertAgentOwned(
  agentId: string,
  ownerId: string
): Promise<void> {
  const found = await db.query.agent.findFirst({
    where: (table) => and(eq(table.id, agentId), eq(table.ownerId, ownerId)),
  })

  if (!found) {
    throw new TRPCError({ code: "BAD_REQUEST", message: "Agent not found" })
  }
}

async function assertTaskOwned(taskId: string, ownerId: string): Promise<void> {
  const found = await db.query.agentTask.findFirst({
    where: (table) => eq(table.id, taskId),
    with: { workspace: true },
  })

  if (!found || found.workspace.ownerId !== ownerId) {
    throw new TRPCError({ code: "BAD_REQUEST", message: "Task not found" })
  }
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
    throw notFound()
  }
}

@Router({ alias: "sessions" })
export class SessionsRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({
    input: agentSessionListInputSchema,
    output: z.array(agentSessionSchema),
  })
  async list(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentSessionListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    return await db.query.agentSession.findMany({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({ input: agentSessionIdInputSchema, output: agentSessionSchema })
  async get(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentSessionIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.id, ownerId)

    const found = await db.query.agentSession.findFirst({
      where: (table) => eq(table.id, input.id),
    })

    if (!found) {
      throw notFound()
    }

    return found
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({
    input: agentSessionCreateInputSchema,
    output: agentSessionSchema,
  })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentSessionCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    if (input.agentId) {
      await assertAgentOwned(input.agentId, ownerId)
    }
    if (input.taskId) {
      await assertTaskOwned(input.taskId, ownerId)
    }

    const [created] = await db
      .insert(agentSession)
      .values({
        agentId: input.agentId ?? null,
        id: crypto.randomUUID(),
        prompt: input.prompt ?? null,
        status: input.status ?? "needs_setup",
        taskId: input.taskId ?? null,
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create session",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentSessionIdInputSchema, output: agentSessionSchema })
  async start(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentSessionIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.id, ownerId)

    const [updated] = await db
      .update(agentSession)
      .set({ endedAt: null, startedAt: new Date(), status: "running" })
      .where(eq(agentSession.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentSessionIdInputSchema, output: agentSessionSchema })
  async stop(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentSessionIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.id, ownerId)

    const [updated] = await db
      .update(agentSession)
      .set({ endedAt: new Date(), status: "stopped" })
      .where(eq(agentSession.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentSessionIdInputSchema, output: agentSessionSchema })
  async retry(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentSessionIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.id, ownerId)

    const [updated] = await db
      .update(agentSession)
      .set({ endedAt: null, startedAt: new Date(), status: "running" })
      .where(eq(agentSession.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({
    input: sessionLogListInputSchema,
    output: z.array(sessionLogSchema),
  })
  async logs(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionLogListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.sessionId, ownerId)

    return await db.query.sessionLog.findMany({
      orderBy: (table, { asc }) => asc(table.createdAt),
      where: (table) => eq(table.sessionId, input.sessionId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: sessionLogAppendInputSchema, output: sessionLogSchema })
  async appendLog(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionLogAppendInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.sessionId, ownerId)

    const [created] = await db
      .insert(sessionLog)
      .values({
        id: crypto.randomUUID(),
        message: input.message,
        sessionId: input.sessionId,
        stream: input.stream ?? "stdout",
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to append session log",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: sessionFinishInputSchema, output: agentSessionSchema })
  async finish(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionFinishInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.id, ownerId)

    const [updated] = await db
      .update(agentSession)
      .set({ endedAt: new Date(), status: input.status })
      .where(eq(agentSession.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({
    input: sessionInstructionInputSchema,
    output: agentSessionSchema,
  })
  async sendInstruction(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof sessionInstructionInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertSessionOwned(input.id, ownerId)

    const session = await db.query.agentSession.findFirst({
      where: (table) => eq(table.id, input.id),
    })

    if (!session) {
      throw notFound()
    }

    await db.insert(sessionLog).values({
      id: crypto.randomUUID(),
      message: `> ${input.instruction}`,
      sessionId: input.id,
      stream: "system",
    })

    if (session.status !== "waiting") {
      return session
    }

    const [updated] = await db
      .update(agentSession)
      .set({ status: "ready" })
      .where(eq(agentSession.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }
}
