CREATE TYPE "public"."workspace_safety_mode" AS ENUM('open', 'guarded');--> statement-breakpoint
ALTER TABLE "workspace" ADD COLUMN "safety_mode" "workspace_safety_mode" DEFAULT 'open' NOT NULL;