import { db } from "@kloucy/db"
import { review } from "@kloucy/db/schema/review"
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

const reviewSchema = z.object({
  action: z.string(),
  artifactId: z.string().nullable(),
  createdAt: z.date(),
  id: z.string(),
  reason: z.string().nullable(),
  sessionId: z.string(),
  workspaceId: z.string(),
})

const reviewIdInputSchema = z.object({
  id: z.string().min(1),
})

const reviewListInputSchema = z.object({
  workspaceId: z.string().min(1),
})

const reviewCreateInputSchema = z.object({
  action: z.string().min(1).optional(),
  artifactId: z.string().min(1).optional(),
  reason: z.string().min(1).optional(),
  sessionId: z.string().min(1),
  workspaceId: z.string().min(1),
})

const reviewUpdateInputSchema = z.object({
  action: z.string().min(1).optional(),
  artifactId: z.string().min(1).nullable().optional(),
  id: z.string().min(1),
  reason: z.string().min(1).nullable().optional(),
})

const reviewRejectInputSchema = z.object({
  id: z.string().min(1),
  reason: z.string().min(1).optional(),
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
  return new TRPCError({ code: "NOT_FOUND", message: "Review not found" })
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

async function assertReviewOwned(
  reviewId: string,
  ownerId: string
): Promise<void> {
  const found = await db.query.review.findFirst({
    where: (table) => eq(table.id, reviewId),
    with: { workspace: true },
  })

  if (!found || found.workspace.ownerId !== ownerId) {
    throw notFound()
  }
}

@Router({ alias: "reviews" })
export class ReviewsRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({ input: reviewListInputSchema, output: z.array(reviewSchema) })
  async list(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof reviewListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    return await db.query.review.findMany({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: reviewCreateInputSchema, output: reviewSchema })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof reviewCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)
    await assertSessionOwned(input.sessionId, ownerId)

    const [created] = await db
      .insert(review)
      .values({
        action: input.action ?? "pending",
        artifactId: input.artifactId ?? null,
        id: crypto.randomUUID(),
        reason: input.reason ?? null,
        sessionId: input.sessionId,
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create review",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: reviewUpdateInputSchema, output: reviewSchema })
  async update(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof reviewUpdateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertReviewOwned(input.id, ownerId)

    const changes: Partial<typeof review.$inferInsert> = {}

    if (input.action !== undefined) {
      changes.action = input.action
    }
    if (input.artifactId !== undefined) {
      changes.artifactId = input.artifactId
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
      .update(review)
      .set(changes)
      .where(eq(review.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: reviewIdInputSchema, output: reviewSchema })
  async approve(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof reviewIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertReviewOwned(input.id, ownerId)

    const [updated] = await db
      .update(review)
      .set({ action: "approved" })
      .where(eq(review.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: reviewRejectInputSchema, output: reviewSchema })
  async reject(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof reviewRejectInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertReviewOwned(input.id, ownerId)

    const changes: Partial<typeof review.$inferInsert> = {
      action: "rejected",
    }

    if (input.reason !== undefined) {
      changes.reason = input.reason
    }

    const [updated] = await db
      .update(review)
      .set(changes)
      .where(eq(review.id, input.id))
      .returning()

    if (!updated) {
      throw notFound()
    }

    return updated
  }
}
