import { TRPCError } from "@trpc/server"
import { Ctx, Query, Router, UseMiddlewares } from "nestjs-trpc"
import { z } from "zod"
import type { Context } from "../context"
import { ProtectedMiddleware } from "../middleware/protected"

const privateDataSchema = z.object({
  message: z.string(),
  user: z.unknown(),
})

const appStatusSchema = z.object({
  name: z.string(),
  status: z.string(),
})

@Router({ alias: "app" })
export class TrpcRouter {
  @Query({ output: appStatusSchema })
  getStatus(): z.infer<typeof appStatusSchema> {
    return {
      name: "Kloucy",
      status: "ok",
    }
  }

  @UseMiddlewares(ProtectedMiddleware)
  @Query({ output: privateDataSchema })
  privateData(@Ctx() ctx: Context): z.infer<typeof privateDataSchema> {
    if (!ctx.session) {
      throw new TRPCError({
        cause: "No session",
        code: "UNAUTHORIZED",
        message: "Authentication required",
      })
    }

    return {
      message: "This is private",
      user: ctx.session.user,
    }
  }
}
