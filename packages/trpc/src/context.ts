import { auth } from "@kloucy/auth"
import { Injectable } from "@nestjs/common"
import { fromNodeHeaders } from "better-auth/node"
import type { ContextOptions, TRPCContext } from "nestjs-trpc"

@Injectable()
export class AppContext implements TRPCContext {
  async create({ req }: ContextOptions) {
    const session = await auth.api.getSession({
      headers: fromNodeHeaders(req.headers),
    })

    return {
      auth: null,
      session,
    }
  }
}

export type Context = Awaited<ReturnType<AppContext["create"]>>
