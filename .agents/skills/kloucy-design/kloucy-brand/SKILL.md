---
name: kloucy-brand
description: Use when creating or reviewing Kloucy brand, product UI, landing pages, docs, marketing materials, visual identity, UX writing, color usage, typography, layout, or design-system decisions. Triggers for Kloucy brand, Kloucy style, agent orchestration, local-agent control, session state, review, pairing, and command-center work.
metadata:
  author: kloucy
  version: "2.0.0"
---

# Kloucy Brand Guidelines

Kloucy is an Agent Development Environment for developers turning scattered local-agent work into controlled, reviewable sessions.

Use this skill to keep Kloucy sharp, calm, pragmatic, and product-led. Kloucy should feel like a focused command center that helps a developer know what agents are available, what each session is doing, what changed, what needs approval, and what action comes next.

## Brand Core

| Area | Direction |
| ---- | --------- |
| Positioning | The orchestration layer for local AI agents |
| Short promise | Agent control for focused developers |
| Product promise | Run local agents, see what changed, and stay in control |
| Audience | Developers, technical founders, indie hackers, agent power users, and small technical teams |
| Primary feeling | Open scattered work; leave with agents under control |

## Core Colors

| Name | Hex |
| ---- | --- |
| Kloucy Black | `#000000` |
| Kloucy White | `#FDFDFD` |

Black and near-white carry the identity. Accent colors communicate state or interaction; they are not decorative brand fills.

Technical source of truth: use `@kloucy/tokens` in code. Web, docs, and Tailwind import `@kloucy/tokens/tailwind.css`; mobile and Uniwind import `@kloucy/tokens/uniwind.css`.

### Semantic Colors

| Scale | Background | Foreground | Usage |
| ----- | ---------- | ---------- | ----- |
| Gray | `#16171AEB` | `#FDFEFFA6` | Structure, hierarchy, inactive agents, and subtle separation |
| Red | `#FF173F2D` | `#FF9592` | Failed sessions, destructive actions, rejected changes, and critical risk |
| Amber | `#FA820022` | `#FFCA16` | Waiting sessions, approvals, caution, degraded state, and uncertainty |
| Green | `#22FF991E` | `#46FEA5D4` | Completed runs, healthy connections, successful pairing, and approved changes |
| Blue | `#0077FF3A` | `#70B8FF` | Interactive focus, links, informational state, and the current selection |

Never use blue as a decorative brand wash. Kloucy remains monochrome-first; blue appears where it explains interaction or information.

## Typography

Use **Geist Variable for everything** in Kloucy: product UI, marketing, docs, headings, body text, labels, buttons, and technical product surfaces unless the content is literal code or terminal output. Native surfaces may use equivalent static Geist cuts when a variable file is unavailable.

| Font | Role |
| ---- | ---- |
| **Geist Variable** | Display, headings, titles, body, labels, navigation, product UI, docs, and marketing copy |
| **System monospace** | Literal code, CLI commands, terminal output, file paths, environment variables, IDs, and agent traces |

### Typography Rules

- Use sentence case for headings, buttons, labels, and navigation.
- Use Geist Variable weights intentionally: Regular, Medium, Semi Bold, and Bold.
- Use Bold only for strong hierarchy, not decoration.
- Never use decorative display fonts, multiple brand fonts, or novelty typefaces.
- Never use monospace for titles, ordinary body copy, labels, or marketing copy.
- Keep letter spacing at `0` unless a specific production asset requires otherwise.
- Prefer concise, stable text that fits compact product UI.
- Do not reduce important text to make an overcrowded layout fit; recompose the layout first.

### Typography Scale

Use named tokens from `@kloucy/tokens`. Avoid raw `text-[...]` values unless measuring a one-off visual proof before promoting it to a token.

#### Dense Product UI

| Style | Weight | Size/line | Use |
| ----- | ------ | --------- | --- |
| `micro` | Regular/Medium | 10/15 | Dense graph ticks, tiny metadata, compact IDs |
| `compact` | Regular/Medium | 11/15.4 | Timestamps, secondary sidebar metadata, line markers |
| `caption` | Regular/Medium | 12/16.8 | Tables, helper text, run metadata, compact product copy |
| `small` | Regular/Medium | 13/19.5 | Navigation, badges, buttons, interactive labels |
| `medium` | Regular/Medium/Semi Bold | 14/21 | Row titles, panel titles, section labels |
| `nav` | Regular/Medium | 14/20 | Desktop navigation links |
| `action` | Medium | 14/20 | Compact actions and primary controls |
| `control` | Regular/Medium | 14/20 | Inputs and placeholders |
| `footer` | Regular/Medium | 14/20 | Footer descriptions and links |
| `large` | Regular/Medium | 15/24 | Mobile paragraphs and standard landing body |
| `body` | Regular/Medium/Semi Bold | 16/24 | Product body where more readability is needed |
| `emphasis` | Medium/Semi Bold | 17/27.2 | Emphasized state or approval copy |

#### Landing Product Demos

Product demos use a stable interface scale at every breakpoint. Responsive behavior recomposes or hides secondary content instead of scaling the UI typography.

| Style | Weight | Size/line | Use |
| ----- | ------ | --------- | --- |
| `demo-metadata` | Regular/Medium | 12/18 | Session IDs, context, badges, and supporting metadata |
| `demo-control` | Medium/Semi Bold | 14/20 | Buttons, inputs, navigation, and row titles |
| `demo-body` | Regular/Medium | 16/24 | Agent explanations and review copy |
| `demo-title` | Semi Bold | 20/28 | Primary title inside a demo surface |
| `demo-stat` | Semi Bold | 24/32 | Supporting run or review metrics |
| `demo-score` | Bold | 56/64 | Primary readiness score |

#### Marketing And Editorial

| Style | Weight | Size/line | Use |
| ----- | ------ | --------- | --- |
| `paragraph-md` | Regular/Medium | 18/28 | Landing paragraphs on medium and large viewports |
| `xlarge` | Regular/Medium | 20/26.6 | Tablet or desktop intro paragraphs |
| `paragraph-xl` | Regular/Medium | 20/30 | Landing paragraphs on wide viewports |
| `section` | Regular/Medium/Semi Bold | 24/31.92 | Mobile section titles or large desktop body |
| `subtitle` | Regular/Medium/Semi Bold | 32/36 | Supporting headlines and quote-scale text |
| `hero` | Medium | 38/41.8 | Compact mobile hero headlines |
| `section-lg` | Medium/Semi Bold/Bold | 40/44 | Tablet section titles |
| `hero-title` | Medium | 46/51 | Landing hero title on mobile and small viewports |
| `title` | Medium/Semi Bold/Bold | 48/48 | Desktop section titles |
| `display` | Medium/Bold | 56/62 | Tablet or desktop hero headlines and large values |
| `display-lg` | Medium/Bold | 64/64 | Wide hero headlines and primary display moments |
| `display-xl` | Bold | 72/72 | Rare maximum-impact desktop headline |
| `hero-title-xl` | Medium | 96/96 | Landing hero title on wide viewports |

#### Responsive Landing Roles

| Role | Base | Medium | Wide |
| ---- | ---- | ------ | ---- |
| Hero title | `hero-title` 46/51 | `display-lg` 64/64 | `hero-title-xl` 96/96 |
| Hero subtitle | `body` 16/24 | `paragraph-md` 18/28 | `paragraph-xl` 20/30 |
| Section title | `subtitle` 32/36 | `section-lg` 40/44 | `title` 48/48 |
| Section paragraph | `body` 16/24 | `paragraph-md` 18/28 | `paragraph-xl` 20/30 |
| Closing title | `section-lg` 40/44 | `title` 48/48 | `display` 56/62 |

Badge, button, navigation, form-control, and footer typography remains stable at its named product token across breakpoints.

## Logo

Do not transfer, derive, or imitate Mavry logo geometry, wordmarks, symbols, favicons, app icons, or naming rules.

There are no approved Kloucy logo, icon, or favicon assets in the repo right now.

Until official Kloucy assets exist:

- Use a text-only wordmark: `Kloucy`.
- Keep the existing wordmark treatment in Inter Variable, semibold `590`; Geist applies to the surrounding product, not retroactively to the logo.
- Use `#FDFDFD` on dark backgrounds and `#000000` on light backgrounds.
- Do not invent logos, icons, favicons, mascots, or abstract symbols.
- Do not reintroduce deleted logo, icon, or favicon files unless the user explicitly provides or requests official assets.

### Clearspace

- Leave at least the wordmark cap height around all sides of the text wordmark.
- Do not place the wordmark inside busy UI screenshots.
- Keep it aligned to product chrome, not floating decoratively.

### Minimum Size

- Web header wordmark: minimum `72px` wide.
- Mobile wordmark: minimum `64px` wide.
- Footer or metadata wordmark: minimum `56px` wide.

### Logo Restrictions

- Do not stretch, skew, rotate, outline, gradient-fill, or texture-fill the wordmark.
- Do not combine the wordmark with unapproved symbols.
- Do not place it on low-contrast backgrounds.
- Do not animate it unless the user explicitly asks for a motion identity exploration.

## Radius, Spacing, And Effects

### Radius

| Token | Value |
| ----- | ----- |
| `xs` | `2px` |
| `sm` | `4px` |
| `md` | `6px` |
| `lg` | `8px` |
| `xl` | `12px` |
| `2xl` | `16px` |
| `3xl` | `24px` |
| `4xl` | `32px` |

Use restrained radii. A radius expresses an interaction boundary or grouping; it is not a default decoration for every object.

### Spacing

Use a `4px` base unit. Build spacing from relationships rather than applying one uniform gap across every section.

### Shadows And Blur

| Token | Value |
| ----- | ----- |
| `shadow-xs` | `0 1px 2px 0 rgb(0 0 0 / 0.24)` |
| `shadow-sm` | `0 8px 24px -16px rgb(0 0 0 / 0.4)` |
| `shadow-md` | `0 16px 48px -28px rgb(0 0 0 / 0.55)` |
| `shadow-lg` | `0 24px 72px -36px rgb(0 0 0 / 0.7)` |
| `blur-glass` | `25px` |

Use blur only when a floating overlay requires context separation. Do not use glass, shadows, or fake depth as general page decoration.

## Gradients

The transferred system includes these production recipes, but they are exceptional utilities rather than a default visual language:

| Name | Value |
| ---- | ----- |
| Font gradient | `linear-gradient(97deg, #ffffff 30%, rgba(255,255,255,0.50) 100%)` |
| Smooth gradient | `linear-gradient(96deg, rgba(255,255,255,0.05) 0%, rgba(255,255,255,0.10) 100%)` |
| Border gradient | `linear-gradient(180deg, rgba(255,255,255,0.10) 0%, rgba(255,255,255,0.05) 100%)` |
| Rainbow border | `linear-gradient(90deg, rgba(2,252,239,0.44) 0%, rgba(255,181,43,0.44) 50%, rgba(160,43,254,0.44) 100%)` |

Do not use gradient text, rainbow borders, glows, blobs, or bokeh in routine product or marketing UI. Use a recipe only for a specific production asset where it communicates hierarchy and the user has approved the direction.

## Design Tokens

Shared product tokens live in `packages/tokens`.

- TypeScript tokens: `@kloucy/tokens`
- Tailwind CSS v4 tokens: `@kloucy/tokens/tailwind.css`
- Uniwind and Expo tokens: `@kloucy/tokens/uniwind.css`

Do not redefine Kloucy colors, typography, radius, spacing, shadows, blur, or semantic states inside apps. Change shared visual decisions in `@kloucy/tokens`, then consume them from web, docs, mobile, desktop adapters, and future surfaces.

## Textures And Background Assets

No approved Kloucy texture, wallpaper, or decorative background assets exist. Do not invent asset URLs or import Mavry assets.

Use black, near-black semantic surfaces, spacing, alignment, and content hierarchy for depth. Light mode may be supported where the product context requires it, but formal Kloucy marketing remains dark-first.

## Reference Quality Bar

Kloucy should reach the product and brand quality level of Vercel, Linear, Resend, and Raycast without copying their logos, exact layouts, claims, colors, animations, or component silhouettes.

- Vercel: crisp hierarchy, technical confidence, restrained surfaces, fast scanning, and product-led proof.
- Linear: focused workflows, calm density, explicit state, strong alignment, and operational elegance.
- Resend: monochrome discipline, sharp typography, clear developer positioning, and minimal ornament.
- Raycast: compact productivity surfaces, decisive shortcuts and actions, polished details, and strong brand memory.

Interpret these as a quality threshold:

- Every screen needs a clear operational idea, not merely a beautiful arrangement.
- Every visual object should explain agent state, progress, risk, review, pairing, or the next action.
- Every section should have a distinct composition instead of repeated cards with different copy.
- Every component should feel precise, quiet, and necessary.
- Kloucy should look intentional beside those products without becoming a clone.

## Layout Patterns

| Name | Description |
| ---- | ----------- |
| Right object scene | Small label and two-line title on the left; a real agent-control object on the right |
| Interface scene | Small label top-left, title bottom-left, real Kloucy UI as the main surface |
| Text-only scene | Short title aligned to the grid with quiet black space |
| Text-only background | Large centered title with restrained monochrome depth |
| Text-only subtle | Small centered statement on a minimal dark background |
| Big number | Large readiness or run value centered with a small explanatory label |

Common patterns:

- Labels remain small and align top-left or top-center.
- Marketing titles use deliberate two-line breaks and never exceed three lines.
- Real session UI, logs, review surfaces, agent registries, and pairing states take priority over abstract art.
- Objects align left, right, or fill the background according to the product story.
- Dark backgrounds use subtle depth and strong alignment rather than ornamental effects.

## Layout And Composition Rules

Kloucy screens are control workspaces, not decorative SaaS pages. Design every screen around the operational question it answers: what agents are doing, what is blocked, what changed, what requires approval, or what action comes next.

### Composition Before Components

- Start with the user question before choosing components.
- Do not default to a heading plus card grid unless the information truly contains peer items.
- Give each important screen one dominant object:
  - Workspace Home: current agent activity, blocked sessions, and the next action.
  - Agent Registry: availability, capability, command, and permission state.
  - Session Board: Ready, Running, Waiting, and Done relationships.
  - Session Timeline: Now, Next, Later, and Not doing sequence.
  - Review: changed files, risk, and explicit approval or rejection.
  - Readiness: blockers, safety state, and what is not required yet.
  - Pairing: QR trust state and the connected mobile device.
- Surround the dominant object with only the context needed to act.
- If every section has equal visual weight, redesign the hierarchy before adding surfaces, color, icons, or borders.

### Operational Geometry

Use geometry to communicate product meaning:

- Agent availability or session state -> explicit position, label, and state marker.
- Execution sequence -> ordered lanes, timeline, or clear vertical progression.
- Readiness -> distance from a threshold plus visible blockers.
- Risk -> proximity to the affected action and stronger semantic contrast.
- Review decisions -> visible separation between proposed, approved, and rejected output.
- Pairing or dependencies -> connection, sequence, or grouped adjacency.

Do not add charts, bars, or diagrams only because data exists. Use them only when they make a control decision faster to understand.

### Grid And Alignment

- Use a consistent page grid across web and desktop product surfaces.
- Align objects to shared edges, baselines, grid lines, or deliberate optical centers.
- Equivalent items share label position, value position, action alignment, row rhythm, and type role.
- Do not strand boards, logs, tables, or diffs in narrow tracks when available width improves scanning.
- Let dense tools use the full content width when lookup, comparison, or live output benefits from it.
- On mobile, recompose before shrinking text or hiding essential state.

### Surfaces And Boundaries

- Treat the page as one continuous workspace by default.
- Use a card, panel, border, or separate surface only when it communicates a real grouping, selected state, warning, interaction zone, or contrast that spacing cannot express.
- Do not wrap every section, metric, session, or artifact in a card.
- Do not put cards inside cards or panels inside panels.
- Do not use borders to compensate for weak hierarchy.
- Prefer spacing, alignment, density shifts, and typography before boxes.
- Keep radii restrained and consistent with Kloucy tokens.

### Rhythm And Density

- Build spacing from relationships rather than one uniform gap.
- Keep headings close to the content they introduce.
- Align labels, values, details, timestamps, and actions consistently across peer rows.
- Use larger separation only when the user moves to a different control area.
- Empty space must help the dominant object stand out; accidental holes and underfilled split layouts are failures.
- Kloucy can be dense because it is operational, but density must stay scannable and calm.

### Screen Structure

- Every section answers a different operational question.
- Remove summaries that repeat the same session state or recommendation.
- Keep one home for each state, decision, or review reason.
- End flows with a clear implication, saved decision, next action, or unresolved blocker.
- Mobile optimizes for pairing, awareness, approval, and short instructions.
- Desktop optimizes for orchestration, logs, diffs, command control, and deep review.
- Web explains the product and shows credible product proof.
- Docs optimize for setup, supported agents, security, pairing, and troubleshooting.

### Responsive Behavior

- Preserve readable type and tappable controls.
- Give grid and flex children `min-width: 0` so content can reflow safely.
- Reflow before truncating or shrinking important text.
- Long logs, diffs, tables, and boards may scroll locally when lookup matters more than stacking everything into cards.
- Light and dark modes preserve the same hierarchy rather than merely inverting colors.

### Motion

- Default to stillness.
- Use motion only to explain a state change, preserve continuity, or confirm an action.
- Do not use motion as decoration, suspense, or personality.
- Never gate reading behind animation.
- Respect reduced-motion preferences.

## Web UI Construction Rules

In `apps/web`, use shadcn/ui through `packages/ui` as the default component foundation. Compose Kloucy-specific screens from shadcn primitives and shared tokens before creating custom generic markup.

### shadcn First

- Use the `shadcn` skill with this Kloucy skill before editing visible web UI.
- Inspect `components.json` when aliases, install paths, base primitives, or monorepo behavior matter.
- Check existing components in `packages/ui/src/components` first.
- Add missing components from `apps/web` with `bunx --bun shadcn@latest add <component>`.
- Import primitives from `@kloucy/ui/components/*` and helpers from `@kloucy/ui/lib/utils`.
- Use semantic CSS variables from `@kloucy/tokens`; do not write raw colors or app-local design variables.

### Component Selection

| Need | Use |
| ---- | --- |
| Primary or secondary action | `Button` |
| Workspace intake, instructions, settings, or auth | `Field`, `Input`, `Textarea`, `Select`, `Checkbox`, `Switch`, `Button` |
| Blocker, failure, risk, or review rationale | `Alert` |
| Session, agent, pairing, or approval state | `Badge` only when the compact boundary adds meaning |
| Agent rows, file changes, reviews, and decision logs | `Table`, `Item`, or composed rows from shadcn primitives |
| A product region with a real boundary | `Card` only when the boundary communicates grouping |
| Unrelated regions | `Separator` |
| Session lanes or product views | `Tabs` or `ToggleGroup` |
| Empty workspace, no agents, no sessions, or no reviews | `Empty` |
| Loading | `Skeleton` or `Spinner` |
| Command flows and confirmation | `Dialog`, `AlertDialog`, `Sheet`, or `Command` |
| Explanatory hover affordance | `Tooltip`, `HoverCard`, or `Popover` |

Custom web components are appropriate for agent orchestration panes, session timelines, log viewers, diff review, QR pairing, readiness objects, and workspace-specific compositions. Do not hand-roll generic buttons, inputs, cards, badges, alerts, dialogs, tabs, tables, forms, skeletons, or tooltips when shadcn provides them.

## Mobile UI Construction Rules

- Use HeroUI Native through semantic variants and compound components.
- Keep native dependencies in `apps/mobile` so Expo autolinking can resolve them.
- Use Uniwind tokens from `@kloucy/tokens/uniwind.css`; do not add raw platform colors.
- Use `onPress`, native navigation, safe-area-aware layouts, and minimum `44px` touch targets.
- Make pairing, active sessions, blockers, and approval actions visually central.
- Mobile continues desktop work; it does not duplicate the complete desktop command center.

## Docs And Desktop Rules

- Docs use the same Geist typography and semantic palette, with prose widths that remain comfortable for setup and security guidance.
- Code and terminal examples use the system monospace stack; prose does not.
- Desktop keeps app chrome quiet and gives logs, session state, diffs, and commands the dominant width.
- Desktop dialogs and overlays may use semantic elevation, but permanent workspace structure should rely on alignment and boundaries rather than shadows.

## AI Design Workflow

Treat the first generated result as a structure draft. Before handing off UI, run these passes:

1. Product pass: name the control question each section or screen answers.
2. Composition pass: remove repeated card grids, equal-weight sections, generic heroes, and fake product visuals.
3. Component pass: replace hand-rolled web primitives with shadcn and mobile primitives with HeroUI Native.
4. Token pass: remove raw colors, one-off spacing, one-off radii, and app-local design variables.
5. Density pass: adjust alignment, rhythm, hierarchy, and whitespace so the screen feels designed rather than evenly distributed.
6. Responsive pass: verify web at desktop, tablet, and mobile widths; verify mobile on representative native sizes.

## AI Generic Detection Checklist

Revise the UI if any of these are true:

- The page could work unchanged for any AI productivity SaaS.
- The hero makes a broad promise but does not show sessions, logs, review, control, or pairing.
- The page is mostly a centered headline, email box, logo row, feature cards, and FAQ.
- Every section uses the same card or grid silhouette.
- The design relies on gradients, glow, blur, icon tiles, pills, or a decorative dashboard to feel polished.
- A product preview looks impressive but does not encode a real agent state or user decision.
- Web code creates local primitives instead of using components already available in `packages/ui`.
- The screen copies a reference brand instead of expressing Kloucy's own product loop.
- The design works in one viewport but loses hierarchy or useful state elsewhere.

## Reject Patterns

Do not ship these patterns in Kloucy:

- Generic centered hero copy followed by a card grid.
- Cards inside cards, panels inside panels, or nested bordered boxes.
- Decorative gradient text, rainbow borders, glows, blobs, bokeh, glass, fake depth, or ornamental textures.
- Decorative grids, stripes, side rails, or noisy abstract backgrounds.
- Repeated metric cards when one composed relationship explains the session state better.
- Pills or rounded capsules for ordinary labels, metadata, or every status.
- Icons as decoration, oversized icons, mixed icon styles, or arbitrary icon tiles.
- A dark rounded rectangle around every chart, log, board, or tool.
- Identical section silhouettes across unrelated product questions.
- Tiny muted copy used to make overcrowded UI fit.
- Logs, tables, or boards compressed into narrow columns while useful width remains empty.
- Color-only meaning for session status, risk, approval, or readiness.
- Fake screenshots, stock imagery, generic AI illustrations, or visuals that do not explain the product.
- Repeated recommendation, summary, rationale, and conclusion blocks that say the same thing.
- Authoring-process copy such as "this section shows" or "we organized this by".
- Visible theme toggles in formal marketing unless the product context requires one.
- Hand-rolled generic primitives when the platform's component system already provides them.
- Marketing that directly imitates Vercel, Linear, Resend, Raycast, or Mavry.

## Design Principles

1. Dark-first design philosophy.
2. Sharp contrast between black and light.
3. Precision and focus over decoration.
4. Accent colors communicate state, not style.
5. Simple, stable, intentional forms.
6. Composition before components.
7. Real product state before abstract visuals.
8. Cross-surface consistency through shared tokens.

## Product Principles

1. Visibility creates trust.
2. Control comes before autonomy.
3. Local workflows are first-class.
4. Sessions are execution, output, changes, approvals, and history—not just chat.
5. Mobile is continuation, not a desktop replacement.
6. Agents work for the user; final control remains with the developer.

## Voice And Tone

Kloucy writes directly, calmly, and without hype.

Use:

- Clear, short operational language.
- Firm guidance without arrogance.
- Practical language for developers.
- Technical specificity when it helps.
- No emojis by default.
- Minimal humor.
- Explicit state, reason, and next action.

Avoid:

- "Unlock limitless AI productivity."
- "AI-powered productivity."
- Motivational filler, vague startup language, or generic autonomy claims.
- Infantilized, overly colorful, corporate, or cold copy.

Preferred copy examples:

```text
This agent is waiting for approval.
```

```text
Three files changed. Review before applying.
```

```text
Mobile paired.
```

```text
This session failed before writing files.
```

## Experience Direction

Kloucy should feel operational and focused:

- Workspace Home answers what agents are doing now.
- Agent detection and capabilities remain explicit.
- Tasks can be captured without automatic execution.
- Running sessions expose live logs and output.
- Waiting sessions explain the required approval or input.
- Reviews preserve what changed and why a decision was made.
- QR pairing feels immediate, legible, and trustworthy.
- Mobile supports awareness, approvals, and short follow-up instructions.
- Desktop remains the command center for deep supervision.
- Dashboards stay useful rather than decorative.
