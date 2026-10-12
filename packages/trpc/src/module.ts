import { Module } from "@nestjs/common"
import { TRPCModule } from "nestjs-trpc"
import { AppContext } from "./context"
import { ProtectedMiddleware } from "./middleware/protected"
import { AgentTasksRouter } from "./routers/agent-tasks.router"
import { AgentsRouter } from "./routers/agents.router"
import { TrpcRouter } from "./routers/app.router"
import { DecisionsRouter } from "./routers/decisions.router"
import { PairingRouter } from "./routers/pairing.router"
import { ReadinessRouter } from "./routers/readiness.router"
import { ReviewsRouter } from "./routers/reviews.router"
import { SessionStepsRouter } from "./routers/session-steps.router"
import { SessionsRouter } from "./routers/sessions.router"
import { WeeklyReviewsRouter } from "./routers/weekly-reviews.router"
import { WorkspacesRouter } from "./routers/workspaces.router"

@Module({
  exports: [
    AppContext,
    ProtectedMiddleware,
    AgentTasksRouter,
    AgentsRouter,
    TrpcRouter,
    DecisionsRouter,
    PairingRouter,
    ReadinessRouter,
    ReviewsRouter,
    SessionStepsRouter,
    SessionsRouter,
    WeeklyReviewsRouter,
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
    DecisionsRouter,
    PairingRouter,
    ReadinessRouter,
    ReviewsRouter,
    SessionStepsRouter,
    SessionsRouter,
    WeeklyReviewsRouter,
    WorkspacesRouter,
  ],
})
export class TrpcModule {}
