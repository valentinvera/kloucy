import { db } from "@kloucy/db"
import { agent } from "@kloucy/db/schema/agent"
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

const agentSchema = z.object({
  capabilities: z.array(z.string()).nullable(),
  commandTemplate: z.string().nullable(),
  createdAt: z.date(),
  defaultWorkspaceId: z.string().nullable(),
  executablePath: z.string().nullable(),
  id: z.string(),
  lastUsedAt: z.date().nullable(),
  name: z.string(),
  ownerId: z.string(),
  permissionMode: z.string().nullable(),
  provider: z.string(),
  status: z.string(),
  updatedAt: z.date(),
})

const agentIdInputSchema = z.object({
  id: z.string().min(1),
})

const agentCreateInputSchema = z.object({
  capabilities: z.array(z.string()).optional(),
  commandTemplate: z.string().min(1).optional(),
  defaultWorkspaceId: z.string().min(1).optional(),
  executablePath: z.string().min(1).optional(),
  name: z.string().min(1),
  permissionMode: z.string().min(1).optional(),
  provider: z.string().min(1),
  status: z.string().min(1).optional(),
})

const agentUpdateInputSchema = z.object({
  capabilities: z.array(z.string()).nullable().optional(),
  commandTemplate: z.string().min(1).nullable().optional(),
  defaultWorkspaceId: z.string().min(1).nullable().optional(),
  executablePath: z.string().min(1).nullable().optional(),
  id: z.string().min(1),
  name: z.string().min(1).optional(),
  permissionMode: z.string().min(1).nullable().optional(),
  provider: z.string().min(1).optional(),
  status: z.string().min(1).optional(),
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
  return new TRPCError({ code: "NOT_FOUND", message: "Agent not found" })
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
      message: "Default workspace not found",
    })
  }
}

@Router({ alias: "agents" })
export class AgentsRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({ output: z.array(agentSchema) })
  async list(@Ctx() ctx: Context) {
    const ownerId = requireUserId(ctx)

    return await db.query.agent.findMany({
      orderBy: (table, { asc }) => asc(table.name),
      where: (table) => eq(table.ownerId, ownerId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentCreateInputSchema, output: agentSchema })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    if (input.defaultWorkspaceId) {
      await assertWorkspaceOwned(input.defaultWorkspaceId, ownerId)
    }

    const [created] = await db
      .insert(agent)
      .values({
        capabilities: input.capabilities ?? null,
        commandTemplate: input.commandTemplate ?? null,
        defaultWorkspaceId: input.defaultWorkspaceId ?? null,
        executablePath: input.executablePath ?? null,
        id: crypto.randomUUID(),
        name: input.name,
        ownerId,
        permissionMode: input.permissionMode ?? null,
        provider: input.provider,
        status: input.status ?? "needs_setup",
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create agent",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentUpdateInputSchema, output: agentSchema })
  async update(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentUpdateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    if (input.defaultWorkspaceId) {
      await assertWorkspaceOwned(input.defaultWorkspaceId, ownerId)
    }

    const changes: Partial<typeof agent.$inferInsert> = {}

    if (input.name !== undefined) {
      changes.name = input.name
    }
    if (input.provider !== undefined) {
      changes.provider = input.provider
    }
    if (input.executablePath !== undefined) {
      changes.executablePath = input.executablePath
    }
    if (input.commandTemplate !== undefined) {
      changes.commandTemplate = input.commandTemplate
    }
    if (input.capabilities !== undefined) {
      changes.capabilities = input.capabilities
    }
    if (input.status !== undefined) {
      changes.status = input.status
    }
    if (input.permissionMode !== undefined) {
      changes.permissionMode = input.permissionMode
    }
    if (input.defaultWorkspaceId !== undefined) {
      changes.defaultWorkspaceId = input.defaultWorkspaceId
    }

    if (Object.keys(changes).length === 0) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "No fields to update",
      })
    }

    const [updated] = await db
      .update(agent)
      .set(changes)
      .where(and(eq(agent.id, input.id), eq(agent.ownerId, ownerId)))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: agentIdInputSchema, output: agentSchema })
  async disable(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof agentIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const [disabled] = await db
      .update(agent)
      .set({ status: "disabled" })
      .where(and(eq(agent.id, input.id), eq(agent.ownerId, ownerId)))
      .returning()

    if (!disabled) {
      throw notFound()
    }

    return disabled
  }
}
