import { auth } from "@kloucy/auth"
import { Controller, Get, Req, UnauthorizedException } from "@nestjs/common"
import { fromNodeHeaders } from "better-auth/node"
import type { Request } from "express"

@Controller("users")
export class UsersController {
  @Get("me")
  async me(@Req() request: Request) {
    const session = await auth.api.getSession({
      headers: fromNodeHeaders(request.headers),
    })

    if (!session) {
      throw new UnauthorizedException("Authentication required")
    }

    return session.user
  }
}
