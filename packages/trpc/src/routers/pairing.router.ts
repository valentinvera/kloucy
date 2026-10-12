import { db } from "@kloucy/db"
import { pairingSession } from "@kloucy/db/schema/pairing-session"
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

const PAIRING_TTL_MS = 5 * 60 * 1000

const pairingSessionSchema = z.object({
  claimedAt: z.date().nullable(),
  createdAt: z.date(),
  deviceName: z.string().nullable(),
  expiresAt: z.date(),
  id: z.string(),
  ownerId: z.string(),
  status: z.string(),
  token: z.string(),
})

const pairingIdInputSchema = z.object({
  id: z.string().min(1),
})

const pairingClaimInputSchema = z.object({
  deviceName: z.string().min(1).optional(),
  token: z.string().min(1),
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
  return new TRPCError({
    code: "NOT_FOUND",
    message: "Pairing session not found",
  })
}

@Router({ alias: "pairing" })
export class PairingRouter {
  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ output: pairingSessionSchema })
  async createQrSession(@Ctx() ctx: Context) {
    const ownerId = requireUserId(ctx)

    const [created] = await db
      .insert(pairingSession)
      .values({
        expiresAt: new Date(Date.now() + PAIRING_TTL_MS),
        id: crypto.randomUUID(),
        ownerId,
        status: "pending",
        token: crypto.randomUUID(),
      })
      .returning()

    if (!created) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: "Failed to create pairing session",
      })
    }

    return created
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: pairingClaimInputSchema, output: pairingSessionSchema })
  async claim(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof pairingClaimInputSchema>
  ) {
    requireUserId(ctx)

    const found = await db.query.pairingSession.findFirst({
      where: (table) => eq(table.token, input.token),
    })

    const isValidClaim =
      found !== undefined &&
      found.status === "pending" &&
      found.expiresAt.getTime() >= Date.now()

    if (!isValidClaim) {
      throw new TRPCError({
        code: "BAD_REQUEST",
        message: "Invalid or expired pairing code",
      })
    }

    const [claimed] = await db
      .update(pairingSession)
      .set({
        claimedAt: new Date(),
        deviceName: input.deviceName ?? null,
        status: "claimed",
      })
      .where(eq(pairingSession.id, found.id))
      .returning()

    if (!claimed) {
      throw notFound()
    }

    return claimed
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({ input: pairingIdInputSchema, output: pairingSessionSchema })
  async getStatus(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof pairingIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const found = await db.query.pairingSession.findFirst({
      where: (table) => and(eq(table.id, input.id), eq(table.ownerId, ownerId)),
    })

    if (!found) {
      throw notFound()
    }

    return found
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Mutation({ input: pairingIdInputSchema, output: pairingSessionSchema })
  async revoke(
    @Ctx() ctx: Context,
    @Input() input: z.infer<typeof pairingIdInputSchema>
  ) {
    const ownerId = requireUserId(ctx)

    const [revoked] = await db
      .update(pairingSession)
      .set({ status: "revoked" })
      .where(
        and(
          eq(pairingSession.id, input.id),
          eq(pairingSession.ownerId, ownerId)
        )
      )
      .returning()

    if (!revoked) {
      throw notFound()
    }

    return revoked
  }
}
