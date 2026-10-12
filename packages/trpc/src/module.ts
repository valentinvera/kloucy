import { Module } from "@nestjs/common"
import { TRPCModule } from "nestjs-trpc"
import { AppContext } from "./context"
import { ProtectedMiddleware } from "./middleware/protected"
import { AgentTasksRouter } from "./routers/agent-tasks.router"
import { AgentsRouter } from "./routers/agents.router"
import { TrpcRouter } from "./routers/app.router"
import { ReviewsRouter } from "./routers/reviews.router"
import { SessionsRouter } from "./routers/sessions.router"
import { WorkspacesRouter } from "./routers/workspaces.router"

@Module({
  exports: [
    AppContext,
    ProtectedMiddleware,
    AgentTasksRouter,
    AgentsRouter,
    TrpcRouter,
    ReviewsRouter,
    SessionsRouter,
    WorkspacesRouter,
  ],
  imports: [
    TRPCModule.forRoot({
      basePath: "/api/trpc",
      context: AppContext,
    }),
  ],
  providers: [
    AppContext,
    ProtectedMiddleware,
    AgentTasksRouter,
    AgentsRouter,
    TrpcRouter,
    ReviewsRouter,
    SessionsRouter,
    WorkspacesRouter,
  ],
})
export class TrpcModule {}
