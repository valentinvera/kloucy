import { Module } from "@nestjs/common"
import { TRPCModule } from "nestjs-trpc"
import { AppContext } from "./context"
import { ProtectedMiddleware } from "./middleware/protected"
import { TrpcRouter } from "./routers/app.router"
import { WorkspacesRouter } from "./routers/workspaces.router"

@Module({
  exports: [AppContext, ProtectedMiddleware, TrpcRouter, WorkspacesRouter],
  imports: [
    TRPCModule.forRoot({
      basePath: "/api/trpc",
      context: AppContext,
    }),
  ],
  providers: [AppContext, ProtectedMiddleware, TrpcRouter, WorkspacesRouter],
})
export class TrpcModule {}
