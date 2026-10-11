import { auth } from "@kloucy/auth"
import { TrpcModule } from "@kloucy/trpc"
import {
  type MiddlewareConsumer,
  Module,
  type NestModule,
  RequestMethod,
} from "@nestjs/common"
import { toNodeHandler } from "better-auth/node"
import { UsersController } from "./users/users.controller"

@Module({
  controllers: [UsersController],
  imports: [TrpcModule],
})
export class AppModule implements NestModule {
  configure(consumer: MiddlewareConsumer): void {
    consumer.apply(toNodeHandler(auth)).forRoutes({
      method: RequestMethod.ALL,
      path: "auth/{*splat}",
    })
  }
}
