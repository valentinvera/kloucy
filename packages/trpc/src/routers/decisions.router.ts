import { db } from "@kloucy/db"
import { decision } from "@kloucy/db/schema/decision"
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

const decisionTypeSchema = z.enum([
  "scope",
  "permission",
  "run",
  "review",
  "reject",
  "revisit",
  "note",
])

const decisionSchema = z.object({
  createdAt: z.date(),
  fromValue: z.string().nullable(),
  id: z.string(),
  reason: z.string().nullable(),
  revisitWhen: z.string().nullable(),
  sessionId: z.string().nullable(),
  summary: z.string().nullable(),
  title: z.string(),
  toValue: z.string().nullable(),
  type: decisionTypeSchema,
  workspaceId: z.string(),
})

const decisionListInputSchema = z.object({
  workspaceId: z.string().min(1),
})

const decisionCreateInputSchema = z.object({
  fromValue: z.string().min(1).optional(),
  reason: z.string().min(1).optional(),
  revisitWhen: z.string().min(1).optional(),
  sessionId: z.string().min(1).optional(),
  summary: z.string().min(1).optional(),
  title: z.string().min(1),
  toValue: z.string().min(1).optional(),
  type: decisionTypeSchema.optional(),
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

@Router({ alias: "decisions" })
export class DecisionsRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({ input: decisionListInputSchema, output: z.array(decisionSchema) })
  async list(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof decisionListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    return await db.query.decision.findMany({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: decisionCreateInputSchema, output: decisionSchema })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof decisionCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    if (input.sessionId) {
      await assertSessionOwned(input.sessionId, ownerId)
    }

    const [created] = await db
      .insert(decision)
      .values({
        fromValue: input.fromValue ?? null,
        id: crypto.randomUUID(),
        reason: input.reason ?? null,
        revisitWhen: input.revisitWhen ?? null,
        sessionId: input.sessionId ?? null,
        summary: input.summary ?? null,
        title: input.title,
        toValue: input.toValue ?? null,
        type: input.type ?? "note",
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create decision",
      })
    }

    return created
  }
}
