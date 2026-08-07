# Project Design System

For all design work in this project, always use the `kloucy-design` skill before creating or reviewing UI, visual identity, layout, UX writing, marketing pages, product screens, docs visuals, or design-system decisions.

Use `kloucy-brand` when applying Kloucy visual identity directly: colors, typography, logo usage, effects, textures, layout patterns, product UI rules, or brand-sensitive copy.

This applies to every Kloucy surface: `apps/web`, `apps/docs`, `apps/mobile`, `apps/desktop`, and future product surfaces. Use it together with the relevant platform-specific guidance, but keep Kloucy's brand, colors, typography, voice, layout patterns, and design principles as the source of truth for visual and UX decisions.

# Product And Brand Context

Before developing product features, flows, data models, API surfaces, agent orchestration behavior, or user-facing behavior, read `PRODUCT_PLAN.md` and keep the implementation aligned with that scope. Product work should follow the MVP boundaries, screens, flows, entities, states, API contracts, milestones, and "this already works" criteria defined there.

Before making decisions about what Kloucy is, who it serves, how it should feel, how it should communicate, or why a product choice matters, read `BRAND_FOUNDATION.md`. Use it as the baseline context for Kloucy's problem, audience, positioning, promise, personality, values, story, and experience principles.

# Where Code Lives

## Apps

- `apps/web`: TanStack Start/Vite React web app. This is the landing and future web product surface for Kloucy.
- `apps/docs`: TanStack Start + Fumadocs documentation app. This is the documentation surface for product, API, agent, and setup docs.
- `apps/mobile`: Expo/React Native app using Expo Router, Uniwind, and HeroUI Native. This is the mobile surface for pairing with desktop, checking agent sessions, and continuing work from a phone.
- `apps/api`: NestJS API app. All API routes live under the `/api` prefix and expose auth, tRPC, and future orchestration endpoints.
- `apps/desktop`: GPUI/Rust desktop app. This is the future local desktop surface for agent orchestration.

## Packages

- `packages/auth`: Better Auth setup and auth-related server utilities shared by the API and clients.
- `packages/db`: Drizzle/Neon database client, schema, and database tooling.
- `packages/env`: Typed environment validation for API, web, and mobile runtimes.
- `packages/tokens`: Shared Kloucy design tokens for TypeScript, Tailwind CSS, and Uniwind.
- `packages/trpc`: NestJS tRPC module, routers, context, middleware, and generated tRPC server code.
- `packages/ui`: Shared web UI styling, shadcn/Tailwind setup, UI helpers, hooks, and reusable web components.
- `packages/ts-config`: Shared TypeScript configs for base, NestJS, Expo, and React library packages.

# Local Commands

Use Bun for all package commands.

## Whole Project From Root

- Install dependencies: `bun install`
- Run all JavaScript apps locally through Turbo with explicit filters; do not use `bun run dev` unless the user asks to include desktop.
- Run only the web app through Turbo: `bun run dev:web`
- Run only the docs app through Turbo: `bun run dev:docs`
- Run only the API app through Turbo: `bun run dev:api`
- Run only the mobile app through Turbo: `bun run dev:mobile`
- Run only the desktop app through Turbo: `bun run dev:desktop`
- Check formatting and linting: `bun run check`
- Check TypeScript across JavaScript packages without desktop: `bunx turbo run check-types --filter=@kloucy/api... --filter=@kloucy/docs... --filter=@kloucy/mobile... --filter=@kloucy/web...`
- Build JavaScript apps/packages without desktop: `bunx turbo run build --filter=@kloucy/api... --filter=@kloucy/docs... --filter=@kloucy/web...`
- Install Playwright Chromium: `bun run browser:install`
- Run Playwright e2e tests: `bun run test:e2e`
- Serve Playwright artifacts: `bun run browser:serve-artifacts`

## `apps/api`

- Run locally: `cd apps/api && bun run dev`
- Build locally: `cd apps/api && bun run build`
- Run built server locally: `cd apps/api && bun run prod`
- Check types: `cd apps/api && bun run check-types`

The API reads `HOST`, `PORT`, `BETTER_AUTH_SECRET`, `BETTER_AUTH_URL`, `CORS_ORIGIN`, and `DATABASE_URL` from env and should serve all routes under `/api`.

## `apps/web`

- Run locally: `cd apps/web && bun run dev`
- Build locally: `cd apps/web && bun run build`
- Run built preview locally: `cd apps/web && bun run prod`

Web uses Vite/TanStack Start and reads `VITE_HOST`, `VITE_PORT`, and `VITE_API_URL` from env.

## `apps/docs`

- Run locally: `cd apps/docs && bun run dev`
- Build locally: `cd apps/docs && bun run build`
- Run built docs locally: `cd apps/docs && bun run start`
- Preview locally: `cd apps/docs && bun run preview`
- Check types: `cd apps/docs && bun run check-types`

Docs uses Vite/TanStack Start/Fumadocs and reads `DOCS_HOST` and `DOCS_PORT` from env.

## `apps/mobile`

- Run locally with Expo: `cd apps/mobile && bun run dev`
- Run Expo without clearing cache: `cd apps/mobile && bun run start`
- Export production bundles: `cd apps/mobile && bun run prod`
- Check types: `cd apps/mobile && bun run check-types`
- Run Android native build locally when needed: `cd apps/mobile && bun run android`
- Run iOS native build locally when needed: `cd apps/mobile && bun run ios`

Mobile uses Expo defaults for local Metro ports unless explicitly configured by Expo. Mobile reads `EXPO_PUBLIC_API_URL` from env.

# Pull Requests

Never create a pull request unless the user explicitly asks for one.

Each PR must have one objective and one responsibility. If the PR body needs wording like "also", "and also", or introduces a second unrelated concern, split the work into separate PRs.

PR titles must use Conventional Commits format:

- `fix(api): guard private trpc route`
- `feat(web): add agent session shell`
- `chore(config): update playwright env passthrough`

PR bodies should be brief and include:

- Problem: what was wrong or missing.
- Solution: how it was fixed or implemented.
- Checks: what was run and whether it passed.
- Impact: user-facing, API, data, migration, deployment, or compatibility impact.

For UI changes that add, remove, or fix visible behavior, include before/after images. If the change involves motion, animation, transitions, gestures, or interaction timing, include a short video instead of only screenshots.

# Commits

Never create a commit unless the user explicitly asks for one.

Commit messages must:

- use Conventional Commits format
- be 100 characters or fewer
- pass Lefthook hooks

Examples:

- `fix(api): guard private trpc route`
- `feat(web): add agent session shell`
- `chore(tokens): add shared radius scale`

Stage files with intent. Do not use a blind `git add .`, and do not stage files one by one when a coherent group should be staged together. Group staged files by the responsibility of the commit.

# Ultracite Code Standards

This project uses **Ultracite**, a zero-config preset that enforces strict code quality standards through automated formatting and linting.

## Quick Reference

- **Format code**: `bun x ultracite fix`
- **Check for issues**: `bun x ultracite check`
- **Diagnose setup**: `bun x ultracite doctor`

Biome (the underlying engine) provides robust linting and formatting. Most issues are automatically fixable.

---

## Core Principles

Write code that is **accessible, performant, type-safe, and maintainable**. Focus on clarity and explicit intent over brevity.

### Type Safety & Explicitness

- Use explicit types for function parameters and return values when they enhance clarity
- Prefer `unknown` over `any` when the type is genuinely unknown
- Use const assertions (`as const`) for immutable values and literal types
- Leverage TypeScript's type narrowing instead of type assertions
- Use meaningful variable names instead of magic numbers - extract constants with descriptive names

### Modern JavaScript/TypeScript

- Use arrow functions for callbacks and short functions
- Prefer `for...of` loops over `.forEach()` and indexed `for` loops
- Use optional chaining (`?.`) and nullish coalescing (`??`) for safer property access
- Prefer template literals over string concatenation
- Use destructuring for object and array assignments
- Use `const` by default, `let` only when reassignment is needed, never `var`

### Async & Promises

- Always `await` promises in async functions - don't forget to use the return value
- Use `async/await` syntax instead of promise chains for better readability
- Handle errors appropriately in async code with try-catch blocks
- Don't use async functions as Promise executors

### React & JSX

- Use function components over class components
- Call hooks at the top level only, never conditionally
- Specify all dependencies in hook dependency arrays correctly
- Use the `key` prop for elements in iterables (prefer unique IDs over array indices)
- Nest children between opening and closing tags instead of passing as props
- Don't define components inside other components
- Use semantic HTML and ARIA attributes for accessibility:
  - Provide meaningful alt text for images
  - Use proper heading hierarchy
  - Add labels for form inputs
  - Include keyboard event handlers alongside mouse events
  - Use semantic elements (`<button>`, `<nav>`, etc.) instead of divs with roles

### Error Handling & Debugging

- Remove `console.log`, `debugger`, and `alert` statements from production code
- Throw `Error` objects with descriptive messages, not strings or other values
- Use `try-catch` blocks meaningfully - don't catch errors just to rethrow them
- Prefer early returns over nested conditionals for error cases

### Code Organization

- Keep functions focused and under reasonable cognitive complexity limits
- Extract complex conditions into well-named boolean variables
- Use early returns to reduce nesting
- Prefer simple conditionals over nested ternary operators
- Group related code together and separate concerns

### Security

- Add `rel="noopener"` when using `target="_blank"` on links
- Avoid `dangerouslySetInnerHTML` unless absolutely necessary
- Don't use `eval()` or assign directly to `document.cookie`
- Validate and sanitize user input

### Performance

- Avoid spread syntax in accumulators within loops
- Use top-level regex literals instead of creating them in loops
- Prefer specific imports over namespace imports
- Avoid barrel files (index files that re-export everything)
- Use proper image components (e.g., Next.js `<Image>`) over `<img>` tags

### Framework-Specific Guidance

**Next.js:**

- Use Next.js `<Image>` component for images
- Use `next/head` or App Router metadata API for head elements
- Use Server Components for async data fetching instead of async Client Components

**React 19+:**

- Use ref as a prop instead of `React.forwardRef`

**Solid/Svelte/Vue/Qwik:**

- Use `class` and `for` attributes (not `className` or `htmlFor`)

---

## Testing

- Write assertions inside `it()` or `test()` blocks
- Avoid done callbacks in async tests - use async/await instead
- Don't use `.only` or `.skip` in committed code
- Keep test suites reasonably flat - avoid excessive `describe` nesting

## When Biome Can't Help

Biome's linter will catch most issues automatically. Focus your attention on:

1. **Business logic correctness** - Biome can't validate your algorithms
2. **Meaningful naming** - Use descriptive names for functions, variables, and types
3. **Architecture decisions** - Component structure, data flow, and API design
4. **Edge cases** - Handle boundary conditions and error states
5. **User experience** - Accessibility, performance, and usability considerations
6. **Documentation** - Add comments for complex logic, but prefer self-documenting code

---

Most formatting and common issues are automatically fixed by Biome. Run `bun x ultracite fix` before committing to ensure compliance.
