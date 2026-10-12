# Kloucy

Kloucy is an Agent Development Environment and orchestration layer for developers who run local AI coding agents. It helps you discover available agents, start a real session in a project, watch progress and output, review what changed, and keep control from a phone.

- **Desktop** is the command center for agent orchestration.
- **Web** is the public and account surface.
- **Mobile** is the continuation and supervision surface.
- **Docs** is the setup and trust surface.

See `PRODUCT_PLAN.md` for the MVP scope and `BRAND_FOUNDATION.md` for the brand intent.

## Applications

- `apps/desktop` — GPUI (Rust) desktop command center.
- `apps/web` — TanStack Start (React) web app.
- `apps/api` — NestJS API: Better Auth, tRPC, and REST, all under `/api`.
- `apps/mobile` — Expo/React Native app (HeroUI Native, Uniwind).
- `apps/docs` — TanStack Start + Fumadocs documentation site.

## Packages

- `packages/db` — Drizzle/Neon schema, migrations, and database client.
- `packages/auth` — Better Auth setup shared by the API and clients.
- `packages/trpc` — NestJS tRPC module, routers, context, and generated server types.
- `packages/ui` — shared web UI (shadcn/Tailwind) primitives and styles.
- `packages/tokens` — shared Kloucy design tokens (Tailwind and Uniwind).
- `packages/env` — typed environment validation for API, web, and mobile runtimes.

## Requirements

- [Bun](https://bun.sh)
- A PostgreSQL database (for example, [Neon](https://neon.tech))

## Setup

```bash
bun install
```

Create the environment files each app needs (`apps/api/.env`, `apps/web/.env`, `apps/docs/.env`, `apps/mobile/.env`). The API reads `DATABASE_URL`, `BETTER_AUTH_SECRET`, `BETTER_AUTH_URL`, `CORS_ORIGIN`, `HOST`, and `PORT`. See `packages/env` for the exact variables.

## Database

Apply the committed migrations, or push the current schema directly:

```bash
cd packages/db
bun run db:migrate   # apply migrations
# or
bun run db:push      # push the current schema
```

`bun run db:generate` creates a new migration from schema changes, and `bun run db:studio` opens Drizzle Studio.

## Development

Run only what you need from the repository root (Turbo):

```bash
bun run dev:api
bun run dev:web
bun run dev:docs
bun run dev:mobile
bun run dev:desktop
```

Each app reads its host and port from its own `.env`. The API serves every route under `/api`.

## Checks

```bash
bun run check                                # Biome formatting and linting
bun run check-types                          # TypeScript types across JS apps
cd apps/desktop && cargo check               # desktop
```

## Repository Structure

```text
kloucy/
├── apps/
│   ├── desktop/   # GPUI (Rust) command center
│   ├── web/       # TanStack Start web app
│   ├── api/       # NestJS API (auth, tRPC, REST)
│   ├── mobile/    # Expo / React Native app
│   └── docs/      # Fumadocs documentation site
├── packages/
│   ├── db/        # Drizzle schema, migrations, client
│   ├── auth/      # Better Auth setup
│   ├── trpc/      # tRPC module, routers, generated types
│   ├── ui/        # shared web UI primitives and styles
│   ├── tokens/    # shared design tokens
│   └── env/       # typed environment validation
└── ...
```

## Contributing

- Use Conventional Commits and one objective per pull request.
- Run `bun run check` and the relevant `check-types` before committing.
