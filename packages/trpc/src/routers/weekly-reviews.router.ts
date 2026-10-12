import { db } from "@kloucy/db"
import { weeklyReview } from "@kloucy/db/schema/weekly-review"
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

const weeklyReviewSchema = z.object({
  blockers: z.array(z.string()).nullable(),
  createdAt: z.date(),
  id: z.string(),
  nextActions: z.array(z.string()).nullable(),
  rejectedChanges: z.array(z.string()).nullable(),
  riskyPatterns: z.array(z.string()).nullable(),
  shipped: z.array(z.string()).nullable(),
  summary: z.string().nullable(),
  workspaceId: z.string(),
})

const weeklyReviewListInputSchema = z.object({
  workspaceId: z.string().min(1),
})

const weeklyReviewCreateInputSchema = z.object({
  blockers: z.array(z.string()).optional(),
  nextActions: z.array(z.string()).optional(),
  rejectedChanges: z.array(z.string()).optional(),
  riskyPatterns: z.array(z.string()).optional(),
  shipped: z.array(z.string()).optional(),
  summary: z.string().min(1).optional(),
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

@Router({ alias: "weeklyReviews" })
export class WeeklyReviewsRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Query({
    input: weeklyReviewListInputSchema,
    output: z.array(weeklyReviewSchema),
  })
  async list(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof weeklyReviewListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    return await db.query.weeklyReview.findMany({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({
    input: weeklyReviewListInputSchema,
    output: weeklyReviewSchema.nullable(),
  })
  async latest(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof weeklyReviewListInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    const found = await db.query.weeklyReview.findFirst({
      orderBy: (table, { desc }) => desc(table.createdAt),
      where: (table) => eq(table.workspaceId, input.workspaceId),
    })

    return found ?? null
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({
    input: weeklyReviewCreateInputSchema,
    output: weeklyReviewSchema,
  })
  async create(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof weeklyReviewCreateInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    await assertWorkspaceOwned(input.workspaceId, ownerId)

    const [created] = await db
      .insert(weeklyReview)
      .values({
        blockers: input.blockers ?? null,
        id: crypto.randomUUID(),
        nextActions: input.nextActions ?? null,
        rejectedChanges: input.rejectedChanges ?? null,
        riskyPatterns: input.riskyPatterns ?? null,
        shipped: input.shipped ?? null,
        summary: input.summary ?? null,
        workspaceId: input.workspaceId,
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create weekly review",
      })
    }

    return created
  }
}
