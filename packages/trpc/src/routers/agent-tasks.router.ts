import { db } from "@kloucy/db"
import { agentSession } from "@kloucy/db/schema/agent-session"
import { agentTask } from "@kloucy/db/schema/agent-task"
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

const agentTaskStatusSchema = z.enum([
  "new",
  "needs_setup",
  "ready",
  "running",
  "later",
  "rejected",
  "converted",
])

const agentTaskSourceSchema = z.enum([
  "desktop",
  "mobile",
  "project_context",
  "user_note",
  "agent_suggestion",
  "other",
])

const agentTaskSchema = z.object({
  assignedAgentId: z.string().nullable(),
  createdAt: z.date(),
  createdSessionId: z.string().nullable(),
  id: z.string(),
  prompt: z.string().nullable(),
  source: agentTaskSourceSchema,
  status: agentTaskStatusSchema,
  title: z.string(),
  updatedAt: z.date(),
  workspaceId: z.string(),
})

const agentTaskIdInputSchema = z.object({
  id: z.string().min(1),
})

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

const agentTaskListInputSchema = z.object({
  workspaceId: z.string().min(1),
})

const agentTaskCreateInputSchema = z.object({
  assignedAgentId: z.string().min(1).optional(),
  prompt: z.string().min(1).optional(),
  source: agentTaskSourceSchema.optional(),
  status: agentTaskStatusSchema.optional(),
  title: z.string().min(1),
  workspaceId: z.string().min(1),
})

const agentTaskUpdateInputSchema = z.object({
  assignedAgentId: z.string().min(1).nullable().optional(),
  id: z.string().min(1),
  prompt: z.string().min(1).nullable().optional(),
  source: agentTaskSourceSchema.optional(),
  status: agentTaskStatusSchema.optional(),
  title: z.string().min(1).optional(),
})

const agentTaskUpdateStatusInputSchema = z.object({
  id: z.string().min(1),
  status: agentTaskStatusSchema,
})

const agentTaskAssignInputSchema = z.object({
  assignedAgentId: z.string().min(1).nullable(),
  id: z.string().min(1),
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
  return new TRPCError({ code: "NOT_FOUND", message: "Agent task not found" })
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
    throw notFound()
  }
}

@Router({ alias: "agentTasks" })
export class AgentTasksRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({ input: agentTaskListInputSchema, output: z.array(agentTaskSchema) })
  async list(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    return await db.query.agentTask.findMany({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentTaskCreateInputSchema, output: agentTaskSchema })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    if (input.assignedAgentId) {
      await assertAgentOwned(input.assignedAgentId, ownerId)
    }

    const [created] = await db
      .insert(agentTask)
      .values({
        assignedAgentId: input.assignedAgentId ?? null,
        id: crypto.randomUUID(),
        prompt: input.prompt ?? null,
        source: input.source ?? "desktop",
        status: input.status ?? "new",
        title: input.title,
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create agent task",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentTaskUpdateInputSchema, output: agentTaskSchema })
  async update(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskUpdateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertTaskOwned(input.id, ownerId)

    if (input.assignedAgentId) {
      await assertAgentOwned(input.assignedAgentId, ownerId)
    }

    const changes: Partial<typeof agentTask.$inferInsert> = {}

    if (input.title !== undefined) {
      changes.title = input.title
    }
    if (input.prompt !== undefined) {
      changes.prompt = input.prompt
    }
    if (input.source !== undefined) {
      changes.source = input.source
    }
    if (input.status !== undefined) {
      changes.status = input.status
    }
    if (input.assignedAgentId !== undefined) {
      changes.assignedAgentId = input.assignedAgentId
    }

    if (Object.keys(changes).length === 0) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "No fields to update",
      })
    }

    const [updated] = await db
      .update(agentTask)
      .set(changes)
      .where(eq(agentTask.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({
    input: agentTaskUpdateStatusInputSchema,
    output: agentTaskSchema,
  })
  async updateStatus(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskUpdateStatusInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertTaskOwned(input.id, ownerId)

    const [updated] = await db
      .update(agentTask)
      .set({ status: input.status })
      .where(eq(agentTask.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentTaskAssignInputSchema, output: agentTaskSchema })
  async assignAgent(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskAssignInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertTaskOwned(input.id, ownerId)

    if (input.assignedAgentId) {
      await assertAgentOwned(input.assignedAgentId, ownerId)
    }

    const [updated] = await db
      .update(agentTask)
      .set({ assignedAgentId: input.assignedAgentId })
      .where(eq(agentTask.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentTaskIdInputSchema, output: agentTaskSchema })
  async delete(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertTaskOwned(input.id, ownerId)

    const [deleted] = await db
      .delete(agentTask)
      .where(eq(agentTask.id, input.id))
      .returning()

    if (!deleted) {
      throw notFound()
    }

    return deleted
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentTaskIdInputSchema, output: agentSessionSchema })
  async startSession(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentTaskIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const task = await db.query.agentTask.findFirst({
      where: (table) => eq(table.id, input.id),
      with: { workspace: true },
    })

    if (!task || task.workspace.ownerId !== ownerId) {
      throw notFound()
    }

    const agentId =
      task.assignedAgentId ?? task.workspace.defaultAgentId ?? null

    const [created] = await db
      .insert(agentSession)
      .values({
        agentId,
        id: crypto.randomUUID(),
        prompt: task.prompt ?? task.title,
        status: agentId ? "ready" : "needs_setup",
        taskId: task.id,
        workspaceId: task.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create session",
      })
    }

    await db
      .update(agentTask)
      .set({ createdSessionId: created.id, status: "converted" })
      .where(eq(agentTask.id, task.id))

    return created
  }
}
