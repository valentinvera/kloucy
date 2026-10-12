CREATE TYPE "public"."agent_session_status" AS ENUM('needs_setup', 'ready', 'running', 'waiting', 'done', 'failed', 'stopped');--> statement-breakpoint
CREATE TYPE "public"."agent_task_source" AS ENUM('desktop', 'mobile', 'project_context', 'user_note', 'agent_suggestion', 'other');--> statement-breakpoint
CREATE TYPE "public"."agent_task_status" AS ENUM('new', 'needs_setup', 'ready', 'running', 'later', 'rejected', 'converted');--> statement-breakpoint
CREATE TYPE "public"."decision_type" AS ENUM('scope', 'permission', 'run', 'review', 'reject', 'revisit', 'note');--> statement-breakpoint
CREATE TYPE "public"."readiness_status" AS ENUM('not_ready', 'almost_ready', 'ready_to_run');--> statement-breakpoint
CREATE TYPE "public"."session_log_stream" AS ENUM('stdout', 'stderr', 'system');--> statement-breakpoint
CREATE TYPE "public"."session_lane" AS ENUM('now', 'next', 'later', 'not_doing');--> statement-breakpoint
CREATE TYPE "public"."workspace_stage" AS ENUM('setup', 'ready', 'active', 'paused', 'archived');--> statement-breakpoint
CREATE TABLE "agent_session" (
	"agent_id" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"ended_at" timestamp,
	"id" text PRIMARY KEY NOT NULL,
	"prompt" text,
	"started_at" timestamp,
	"status" "agent_session_status" DEFAULT 'needs_setup' NOT NULL,
	"task_id" text,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "agent_task" (
	"assigned_agent_id" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"created_session_id" text,
	"id" text PRIMARY KEY NOT NULL,
	"prompt" text,
	"source" "agent_task_source" DEFAULT 'desktop' NOT NULL,
	"status" "agent_task_status" DEFAULT 'new' NOT NULL,
	"title" text NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "agent" (
	"capabilities" text[],
	"command_template" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"default_workspace_id" text,
	"executable_path" text,
	"id" text PRIMARY KEY NOT NULL,
	"last_used_at" timestamp,
	"name" text NOT NULL,
	"owner_id" text NOT NULL,
	"permission_mode" text,
	"provider" text NOT NULL,
	"status" text DEFAULT 'needs_setup' NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "account" (
	"access_token" text,
	"access_token_expires_at" timestamp,
	"account_id" text NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"id_token" text,
	"password" text,
	"provider_id" text NOT NULL,
	"refresh_token" text,
	"refresh_token_expires_at" timestamp,
	"scope" text,
	"updated_at" timestamp NOT NULL,
	"user_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "session" (
	"created_at" timestamp DEFAULT now() NOT NULL,
	"expires_at" timestamp NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"ip_address" text,
	"token" text NOT NULL,
	"updated_at" timestamp NOT NULL,
	"user_agent" text,
	"user_id" text NOT NULL,
	CONSTRAINT "session_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "user" (
	"created_at" timestamp DEFAULT now() NOT NULL,
	"email" text NOT NULL,
	"email_verified" boolean DEFAULT false NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"image" text,
	"name" text NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "user_email_unique" UNIQUE("email")
);
--> statement-breakpoint
CREATE TABLE "verification" (
	"created_at" timestamp DEFAULT now() NOT NULL,
	"expires_at" timestamp NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"identifier" text NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	"value" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "decision" (
	"created_at" timestamp DEFAULT now() NOT NULL,
	"from_value" text,
	"id" text PRIMARY KEY NOT NULL,
	"reason" text,
	"revisit_when" text,
	"session_id" text,
	"summary" text,
	"title" text NOT NULL,
	"to_value" text,
	"type" "decision_type" DEFAULT 'note' NOT NULL,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "pairing_session" (
	"claimed_at" timestamp,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"device_name" text,
	"expires_at" timestamp NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"owner_id" text NOT NULL,
	"status" text DEFAULT 'pending' NOT NULL,
	"token" text NOT NULL,
	CONSTRAINT "pairing_session_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "readiness_snapshot" (
	"blockers" text[],
	"created_at" timestamp DEFAULT now() NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"not_required_yet" text[],
	"score" integer NOT NULL,
	"session_id" text,
	"status" "readiness_status" NOT NULL,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "review" (
	"action" text DEFAULT 'pending' NOT NULL,
	"artifact_id" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"reason" text,
	"session_id" text NOT NULL,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "session_log" (
	"created_at" timestamp DEFAULT now() NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"message" text NOT NULL,
	"session_id" text NOT NULL,
	"stream" "session_log_stream" DEFAULT 'stdout' NOT NULL
);
--> statement-breakpoint
CREATE TABLE "session_step" (
	"created_at" timestamp DEFAULT now() NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"lane" "session_lane" DEFAULT 'now' NOT NULL,
	"reason" text,
	"session_id" text NOT NULL,
	"sort_order" integer DEFAULT 0 NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "weekly_review" (
	"blockers" text[],
	"created_at" timestamp DEFAULT now() NOT NULL,
	"id" text PRIMARY KEY NOT NULL,
	"next_actions" text[],
	"rejected_changes" text[],
	"risky_patterns" text[],
	"shipped" text[],
	"summary" text,
	"workspace_id" text NOT NULL
);
--> statement-breakpoint
CREATE TABLE "workspace" (
	"archived_at" timestamp,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"default_agent_id" text,
	"description" text,
	"id" text PRIMARY KEY NOT NULL,
	"ignored_paths" text[],
	"local_path" text NOT NULL,
	"mobile_pairing_enabled" boolean DEFAULT false NOT NULL,
	"name" text NOT NULL,
	"owner_id" text NOT NULL,
	"package_manager" text,
	"project_type" text,
	"runtime" text,
	"safe_commands" text[],
	"stage" "workspace_stage" DEFAULT 'setup' NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "agent_session" ADD CONSTRAINT "agent_session_agent_id_agent_id_fk" FOREIGN KEY ("agent_id") REFERENCES "public"."agent"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "agent_session" ADD CONSTRAINT "agent_session_task_id_agent_task_id_fk" FOREIGN KEY ("task_id") REFERENCES "public"."agent_task"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "agent_session" ADD CONSTRAINT "agent_session_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "agent_task" ADD CONSTRAINT "agent_task_assigned_agent_id_agent_id_fk" FOREIGN KEY ("assigned_agent_id") REFERENCES "public"."agent"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "agent_task" ADD CONSTRAINT "agent_task_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "agent" ADD CONSTRAINT "agent_default_workspace_id_workspace_id_fk" FOREIGN KEY ("default_workspace_id") REFERENCES "public"."workspace"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "agent" ADD CONSTRAINT "agent_owner_id_user_id_fk" FOREIGN KEY ("owner_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "account" ADD CONSTRAINT "account_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session" ADD CONSTRAINT "session_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "decision" ADD CONSTRAINT "decision_session_id_agent_session_id_fk" FOREIGN KEY ("session_id") REFERENCES "public"."agent_session"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "decision" ADD CONSTRAINT "decision_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "pairing_session" ADD CONSTRAINT "pairing_session_owner_id_user_id_fk" FOREIGN KEY ("owner_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "readiness_snapshot" ADD CONSTRAINT "readiness_snapshot_session_id_agent_session_id_fk" FOREIGN KEY ("session_id") REFERENCES "public"."agent_session"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "readiness_snapshot" ADD CONSTRAINT "readiness_snapshot_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "review" ADD CONSTRAINT "review_session_id_agent_session_id_fk" FOREIGN KEY ("session_id") REFERENCES "public"."agent_session"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "review" ADD CONSTRAINT "review_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session_log" ADD CONSTRAINT "session_log_session_id_agent_session_id_fk" FOREIGN KEY ("session_id") REFERENCES "public"."agent_session"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session_step" ADD CONSTRAINT "session_step_session_id_agent_session_id_fk" FOREIGN KEY ("session_id") REFERENCES "public"."agent_session"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session_step" ADD CONSTRAINT "session_step_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "weekly_review" ADD CONSTRAINT "weekly_review_workspace_id_workspace_id_fk" FOREIGN KEY ("workspace_id") REFERENCES "public"."workspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "workspace" ADD CONSTRAINT "workspace_owner_id_user_id_fk" FOREIGN KEY ("owner_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "agent_session_workspaceId_idx" ON "agent_session" USING btree ("workspace_id");--> statement-breakpoint
CREATE INDEX "agent_task_workspaceId_idx" ON "agent_task" USING btree ("workspace_id");--> statement-breakpoint
CREATE INDEX "agent_ownerId_idx" ON "agent" USING btree ("owner_id");--> statement-breakpoint
CREATE INDEX "account_userId_idx" ON "account" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "session_userId_idx" ON "session" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "verification_identifier_idx" ON "verification" USING btree ("identifier");--> statement-breakpoint
CREATE INDEX "decision_workspaceId_idx" ON "decision" USING btree ("workspace_id");--> statement-breakpoint
CREATE INDEX "pairing_session_ownerId_idx" ON "pairing_session" USING btree ("owner_id");--> statement-breakpoint
CREATE INDEX "readiness_snapshot_workspaceId_idx" ON "readiness_snapshot" USING btree ("workspace_id");--> statement-breakpoint
CREATE INDEX "review_workspaceId_idx" ON "review" USING btree ("workspace_id");--> statement-breakpoint
CREATE INDEX "review_sessionId_idx" ON "review" USING btree ("session_id");--> statement-breakpoint
CREATE INDEX "session_log_sessionId_idx" ON "session_log" USING btree ("session_id");--> statement-breakpoint
CREATE INDEX "session_step_sessionId_idx" ON "session_step" USING btree ("session_id");--> statement-breakpoint
CREATE INDEX "weekly_review_workspaceId_idx" ON "weekly_review" USING btree ("workspace_id");--> statement-breakpoint
CREATE INDEX "workspace_ownerId_idx" ON "workspace" USING btree ("owner_id");