import { db } from "@kloucy/db"
import { workspace } from "@kloucy/db/schema/workspace"
import { TRPCError } from "@trpc/server"
import { and, eq, ne } from "drizzle-orm"
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

const workspaceStageSchema = z.enum([
  "setup",
  "ready",
  "active",
  "paused",
  "archived",
])

const workspaceSchema = z.object({
  archivedAt: z.date().nullable(),
  createdAt: z.date(),
  defaultAgentId: z.string().nullable(),
  description: z.string().nullable(),
  id: z.string(),
  ignoredPaths: z.array(z.string()).nullable(),
  localPath: z.string(),
  mobilePairingEnabled: z.boolean(),
  name: z.string(),
  ownerId: z.string(),
  packageManager: z.string().nullable(),
  projectType: z.string().nullable(),
  runtime: z.string().nullable(),
  safeCommands: z.array(z.string()).nullable(),
  stage: workspaceStageSchema,
  updatedAt: z.date(),
})

const workspaceIdInputSchema = z.object({
  id: z.string().min(1),
})

const workspaceCreateInputSchema = z.object({
  description: z.string().min(1).optional(),
  ignoredPaths: z.array(z.string()).optional(),
  localPath: z.string().min(1),
  mobilePairingEnabled: z.boolean().optional(),
  name: z.string().min(1),
  packageManager: z.string().min(1).optional(),
  projectType: z.string().min(1).optional(),
  runtime: z.string().min(1).optional(),
  safeCommands: z.array(z.string()).optional(),
})

const workspaceUpdateInputSchema = z.object({
  defaultAgentId: z.string().min(1).nullable().optional(),
  description: z.string().min(1).nullable().optional(),
  id: z.string().min(1),
  ignoredPaths: z.array(z.string()).nullable().optional(),
  localPath: z.string().min(1).optional(),
  mobilePairingEnabled: z.boolean().optional(),
  name: z.string().min(1).optional(),
  packageManager: z.string().min(1).nullable().optional(),
  projectType: z.string().min(1).nullable().optional(),
  runtime: z.string().min(1).nullable().optional(),
  safeCommands: z.array(z.string()).nullable().optional(),
  stage: workspaceStageSchema.optional(),
})

const workspaceIntakeInputSchema = z.object({
  defaultAgentId: z.string().min(1).nullable().optional(),
  id: z.string().min(1),
  ignoredPaths: z.array(z.string()).optional(),
  mobilePairingEnabled: z.boolean().optional(),
  packageManager: z.string().min(1).optional(),
  projectType: z.string().min(1).optional(),
  runtime: z.string().min(1).optional(),
  safeCommands: z.array(z.string()).optional(),
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
  return new TRPCError({ code: "NOT_FOUND", message: "Workspace not found" })
}

async function assertAgentOwned(
  agentId: string,
  ownerId: string
): Promise<void> {
  const found = await db.query.agent.findFirst({
    where: (table) => and(eq(table.id, agentId), eq(table.ownerId, ownerId)),
  })

  if (!found) {
    throw new TRPCError({
      code: "BAD_REQUEST",
      message: "Default agent not found",
    })
  }
}

@Router({ alias: "workspaces" })
export class WorkspacesRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({ output: z.array(workspaceSchema) })
  async list(@Ctx() ctx: Context) {
    const ownerId = requireUserId(ctx)

    return await db.query.workspace.findMany({
      orderBy: (table, { desc }) => desc(table.updatedAt),
      where: (table) =>
        and(eq(table.ownerId, ownerId), ne(table.stage, "archived")),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({ input: workspaceIdInputSchema, output: workspaceSchema })
  async get(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof workspaceIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const found = await db.query.workspace.findFirst({
      where: (table) => and(eq(table.id, input.id), eq(table.ownerId, ownerId)),
    })

    if (!found) {
      throw notFound()
    }

    return found
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: workspaceCreateInputSchema, output: workspaceSchema })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof workspaceCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const [created] = await db
      .insert(workspace)
      .values({
        description: input.description ?? null,
        id: crypto.randomUUID(),
        ignoredPaths: input.ignoredPaths ?? null,
        localPath: input.localPath,
        mobilePairingEnabled: input.mobilePairingEnabled ?? false,
        name: input.name,
        ownerId,
        packageManager: input.packageManager ?? null,
        projectType: input.projectType ?? null,
        runtime: input.runtime ?? null,
        safeCommands: input.safeCommands ?? null,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create workspace",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: workspaceUpdateInputSchema, output: workspaceSchema })
  async update(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof workspaceUpdateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const changes: Partial<typeof workspace.$inferInsert> = {}

    if (input.name !== undefined) {
      changes.name = input.name
    }
    if (input.localPath !== undefined) {
      changes.localPath = input.localPath
    }
    if (input.description !== undefined) {
      changes.description = input.description
    }
    if (input.projectType !== undefined) {
      changes.projectType = input.projectType
    }
    if (input.defaultAgentId !== undefined) {
      changes.defaultAgentId = input.defaultAgentId
    }
    if (input.packageManager !== undefined) {
      changes.packageManager = input.packageManager
    }
    if (input.runtime !== undefined) {
      changes.runtime = input.runtime
    }
    if (input.safeCommands !== undefined) {
      changes.safeCommands = input.safeCommands
    }
    if (input.ignoredPaths !== undefined) {
      changes.ignoredPaths = input.ignoredPaths
    }
    if (input.mobilePairingEnabled !== undefined) {
      changes.mobilePairingEnabled = input.mobilePairingEnabled
    }
    if (input.stage !== undefined) {
      changes.stage = input.stage
    }

    if (Object.keys(changes).length === 0) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "No fields to update",
      })
    }

    const [updated] = await db
      .update(workspace)
      .set(changes)
      .where(and(eq(workspace.id, input.id), eq(workspace.ownerId, ownerId)))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: workspaceIdInputSchema, output: workspaceSchema })
  async archive(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof workspaceIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const [archived] = await db
      .update(workspace)
      .set({ archivedAt: new Date(), stage: "archived" })
      .where(and(eq(workspace.id, input.id), eq(workspace.ownerId, ownerId)))
      .returning()

    if (!archived) {
      throw notFound()
    }

    return archived
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: workspaceIntakeInputSchema, output: workspaceSchema })
  async completeIntake(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof workspaceIntakeInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    if (input.defaultAgentId) {
      await assertAgentOwned(input.defaultAgentId, ownerId)
    }

    const changes: Partial<typeof workspace.$inferInsert> = { stage: "ready" }

    if (input.defaultAgentId !== undefined) {
      changes.defaultAgentId = input.defaultAgentId
    }
    if (input.projectType !== undefined) {
      changes.projectType = input.projectType
    }
    if (input.packageManager !== undefined) {
      changes.packageManager = input.packageManager
    }
    if (input.runtime !== undefined) {
      changes.runtime = input.runtime
    }
    if (input.safeCommands !== undefined) {
      changes.safeCommands = input.safeCommands
    }
    if (input.ignoredPaths !== undefined) {
      changes.ignoredPaths = input.ignoredPaths
    }
    if (input.mobilePairingEnabled !== undefined) {
      changes.mobilePairingEnabled = input.mobilePairingEnabled
    }

    const [updated] = await db
      .update(workspace)
      .set(changes)
      .where(and(eq(workspace.id, input.id), eq(workspace.ownerId, ownerId)))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }
}
