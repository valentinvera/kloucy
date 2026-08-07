---
name: kloucy-brand
description: "Use when applying Kloucy visual identity: colors, typography, logo policy, effects, textures, backgrounds, layout patterns, and product UI rules."
metadata:
  author: kloucy
  version: "1.0.0"
---

# Kloucy Brand Guidelines

Kloucy uses a dark, precise product visual system: layered product surfaces, measured typography, restrained indigo accents, compact controls, subtle borders, and real product UI as the main visual signal.

## Core Colors

Use dark mode as the default Kloucy brand surface.

| Color | Value | Usage |
|-------|-------|-------|
| Background Primary | `#08090A` | App/page canvas |
| Background Secondary | `#1C1C1F` | Secondary bands and elevated dark areas |
| Background Tertiary | `#232326` | Hover states and low emphasis controls |
| Background Quaternary | `#28282C` | Active states and stronger fills |
| Marketing Background | `#010102` | Marketing hero and high contrast sections |
| Panel | `#0F1011` | App panels, command menus, framed product surfaces |
| Level 0 | `#08090A` | Base app level |
| Level 1 | `#0F1011` | Primary panel level |
| Level 2 | `#141516` | Nested panel level |
| Level 3 | `#191A1B` | Selected and hover row level |
| Text Primary | `#F7F8F8` | Primary headings and active labels |
| Text Secondary | `#D0D6E0` | Body copy and normal labels |
| Text Tertiary | `#8A8F98` | Metadata, helper text, secondary navigation |
| Text Quaternary | `#62666D` | Disabled text, separators, timestamps |
| Border Primary | `#23252A` | Primary panel borders |
| Border Secondary | `#34343A` | Stronger separators |
| Border Tertiary | `#3E3E44` | Highest emphasis border |
| Border Translucent | `#FFFFFF0D` | Hairline overlays |
| Border Translucent Strong | `#FFFFFF14` | Visible translucent rules |
| Brand | `#5E6AD2` | Primary actions and Kloucy brand accent |
| Accent | `#7170FF` | Focus rings, links, selection accents |
| Accent Hover | `#828FFF` | Hovered links and active accents |
| Accent Tint | `#18182F` | Indigo-tinted backgrounds |
| Link Primary | `#828FFF` | Text links on dark surfaces |
| Link Hover | `#FFFFFF` | Link hover on dark surfaces |
| Brand Text | `#FFFFFF` | Text on brand backgrounds |
| Invert Button | `#E5E5E6` | Inverted primary surfaces |

## Light Mode Colors

Only use light mode when the target surface requires it. Kloucy should still feel product-focused and quiet.

| Color | Value |
|-------|-------|
| Background Primary | `#FFFFFF` |
| Background Secondary | `#F9F8F9` |
| Background Tertiary | `#F4F2F4` |
| Background Quaternary | `#EEEDEF` |
| Background Quinary | `#E9E8EA` |
| Text Primary | `#282A30` |
| Text Secondary | `#3C4149` |
| Text Tertiary | `#6F6E77` |
| Text Quaternary | `#86848D` |
| Border Primary | `#E9E8EA` |
| Border Secondary | `#E4E2E4` |
| Border Tertiary | `#DCDBDD` |
| Brand | `#7070FF` |
| Accent | `#7170FF` |
| Accent Hover | `#8989F0` |
| Accent Tint | `#F1F1FF` |

## Semantic Colors

Keep semantic states muted and compatible with the dark Kloucy surface.

| Semantic | Foreground | Background | Border | Usage |
|----------|------------|------------|--------|-------|
| Info | `#AAB1FF` | `#18182F` | `#5E6AD2` | Agent status, neutral notices, selected states |
| Success | `#6DD19B` | `#10251A` | `#2F8A58` | Completed runs, healthy checks |
| Warning | `#F0B864` | `#2B2111` | `#A36A1D` | Requires attention, degraded status |
| Critical | `#FF8F92` | `#2A1114` | `#C43B40` | Failed runs, destructive actions |
| Pending | `#8A8F98` | `#141516` | `#34343A` | Queued or idle states |

## Typography

Use the same typographic stance across web, mobile, desktop, docs, and generated visuals.

| Role | Font |
|------|------|
| UI and body | `"Inter Variable", "SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif` |
| Code and terminal | `"Berkeley Mono", ui-monospace, "SF Mono", Menlo, monospace` |
| Emoji fallback | `"Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji"` |

## Typography Rules

- Use Inter Variable for all product UI, marketing UI, docs UI, and body text.
- Use Berkeley Mono or the platform monospace stack only for code, logs, terminal output, IDs, keyboard shortcuts, and agent traces.
- Use sentence case for headings, labels, buttons, and navigation.
- Keep letter spacing at `0` for normal UI text. Use the negative tracking values below only for display/title tokens.
- Use weights `400`, `510`, `590`, and `680`. Avoid heavier decorative weights.
- Keep copy short, precise, and operational. Kloucy should feel like software, not a brochure.
- Do not use decorative serif display type unless the user explicitly asks for a one-off marketing exploration.

## Typography Scale

| Token | Size | Line Height | Letter Spacing | Usage |
|-------|------|-------------|----------------|-------|
| Tiny | `10px` | `1.5` | `-0.015em` | Dense metadata and graph labels |
| Micro | `12px` | `1.4` | `0` | Captions, badges, secondary table labels |
| Mini | `13px` | `1.5` | `-0.01em` | Navigation, sidebars, footer links |
| Small | `14px` | `21px` | `-0.013em` | Form labels, menu items, helper text |
| Regular | `15px` | `1.6` | `-0.011em` | Default body and product UI text |
| Large | `17px` | `1.6` | `0` | Intro copy and roomy body text |
| Title 1 | `17px` | `1.4` | `-0.012em` | Compact panel titles |
| Title 2 | `20px` | `1.33` | `-0.012em` | Card and section titles |
| Title 3 | `24px` | `1.33` | `-0.012em` | Page subheadings |
| Title 4 | `32px` | `1.125` | `-0.022em` | Page headings |
| Title 5 | `40px` | `1.1` | `-0.022em` | Marketing section headings |
| Title 6 | `48px` | `1` | `-0.022em` | Small hero headings |
| Title 7 | `56px` | `1.1` | `-0.022em` | Hero headings |
| Title 8 | `64px` | `1.06` | `-0.022em` | Large desktop hero headings |
| Title 9 | `72px` | `1` | `-0.022em` | Rare oversized launch moments |

## Radius, Borders, Shadows

| Token | Value | Usage |
|-------|-------|-------|
| Radius 4 | `4px` | Keyboard keys, tiny controls |
| Radius 6 | `6px` | Menu items, compact rows |
| Radius 8 | `8px` | Buttons, cards, selected rows |
| Radius 12 | `12px` | Dialogs, command menu, major panels |
| Radius 16 | `16px` | Large framed product previews only |
| Radius 24 | `24px` | Rare marketing/device frames |
| Rounded | `9999px` | Pills and circular controls |
| Hairline | `1px` | Standard borders |
| Low Shadow | `0px 2px 4px #0000001A` | Low elevation |
| Medium Shadow | `0px 4px 24px #0003` | Popovers and menus |
| High Shadow | `0px 7px 32px #00000059` | Dialogs and command palette |

## Components

### Buttons

- Mini: `24px` height, `12px` text, `12px` icon, `0 10px` padding, `4px` gap.
- Small: `32px` height, `13px` text, `16px` icon, `0 12px` padding, `8px` gap.
- Medium: `40px` height, `13px` text, `16px` icon, `0 14px` padding, `8px` gap.
- Default: `40px` height, `15px` text, `18px` icon, `0 16px` padding, `6px` gap.
- Large: `44px` height, `16px` text, `18px` icon, `0 20px` padding, `6px` gap.
- Primary buttons use `#5E6AD2` with white text.
- Secondary buttons use translucent dark fills with inset hairlines.
- Ghost buttons start tertiary and become primary on hover.
- Active press should scale to `0.97` with a short `160ms` ease.

### Command Menus

- Max width: `720px`.
- Top position: about `13vh`.
- Input height: `46px`.
- Border radius: `12px`.
- Background: `#0F1011`.
- Border: `#FFFFFF0D` or `#23252A`.
- Overlay: `#000000D9`.
- Selected item: `#191A1B`, `8px` radius.

### Keyboard Shortcuts

- Small: `16px` min size, `10px` text.
- Normal: `20px` min size, `13px` text.
- Radius: `4px`.
- Use muted text and subtle border on normal keys.

## Logo

There are no approved Kloucy logo, icon, or favicon assets in the repo right now.

Until official assets exist:

- Use a text-only wordmark: `Kloucy`.
- Set the wordmark in Inter Variable, semibold `590`.
- Use `#F7F8F8` on dark backgrounds and `#282A30` on light backgrounds.
- Do not invent logos, icons, favicons, mascots, or abstract symbols.
- Do not reintroduce deleted logo/icon/favicon files unless the user explicitly provides or requests official assets.

## Clearspace

- Leave at least the wordmark cap height around all sides of the text wordmark.
- Do not place the wordmark inside busy UI screenshots.
- Keep it aligned to product chrome, not floating decoratively.

## Minimum Size

- Web header wordmark: minimum `72px` wide.
- Mobile wordmark: minimum `64px` wide.
- Footer or metadata wordmark: minimum `56px` wide.

## Logo Restrictions

- Do not stretch, skew, rotate, outline, gradient-fill, or texture-fill the wordmark.
- Do not combine the wordmark with unapproved symbols.
- Do not place it on low contrast backgrounds.
- Do not animate it unless the user explicitly asks for a motion identity exploration.

## Product UI Element

The main Kloucy visual asset is the product interface itself:

- Agent sessions
- System-installed agents
- QR pairing between desktop and mobile
- Live terminal/log output
- Task state transitions
- Workspace, inbox, reviews, and orchestration panes

Use real UI states over abstract illustrations.

## Gradients

Use gradients sparingly and functionally.

| Gradient | Value | Usage |
|----------|-------|-------|
| Brand Glow | `radial-gradient(circle, rgba(94, 106, 210, 0.22) 0%, transparent 56%)` | Focused hero or product glow |
| White Edge | `radial-gradient(circle, rgba(247, 248, 248, 0.08) 0%, transparent 50%)` | Panel highlight |
| Panel Fade | `background: #0F1011; box-shadow: inset 0 -80px 120px #08090A;` | Background depth |
| Border Fade | `border-color: #FFFFFF0D; box-shadow: inset 0 1px #FFFFFF14;` | Thin panel borders |

Do not use colorful gradient blobs, bokeh, or decorative orbs.

## Effects

### Glass

- Use `backdrop-filter: blur(4px)` for secondary controls.
- Use `backdrop-filter: blur(8px)` for glass hover states.
- Use `backdrop-filter: blur(24px)` only for floating tooltips or overlays.
- Pair glass with `#FFFFFF0D` border or `#FFFFFF1A` for strong glass.

### Edge Highlights

- Add a subtle top-left or top-edge highlight to framed product surfaces.
- Use translucent white at `#FFFFFF0F` or lower.
- Keep highlights tied to the border geometry.

### Motion

- Default transition duration: `160ms`.
- Dialog scale/fade: `175ms`.
- Tooltip open/close: `120ms`.
- Preferred easing: `cubic-bezier(.25, .46, .45, .94)` for standard UI and `cubic-bezier(.23, 1, .32, 1)` for entrances.
- Press interactions may scale to `0.97`.

## Textures

- Avoid visible paper, grain, fabric, or decorative textures.
- If texture is needed, use only extremely subtle noise over dark product screenshots.
- Prefer borders, shadows, and depth layers over texture.

## Backgrounds

- Default app background: `#08090A`.
- Marketing background: `#010102`.
- Product preview panels: `#0F1011`.
- Nested rows and selected states: `#191A1B`.
- Use masks or fades only to guide attention to real UI.

## Layout Patterns

### Product App Shell

- Left sidebar, main work area, right detail panel when needed.
- Dense but readable spacing.
- Primary navigation uses `13px` text and muted tertiary color.
- Selected rows use level 3 background with 6-8px radius.
- Keep dividers hairline and low contrast.

### Agent Orchestration View

- Show agents, sessions, tasks, logs, and connection state as first-class entities.
- Use split panes for conversation, terminal output, file changes, and run metadata.
- Preserve scanability with compact tables, status chips, and consistent timestamps.
- Use indigo only for focus, active state, primary action, or selected agent.

### Landing Pages

- Make the product UI the first-viewport signal.
- Use a dark hero with concise copy and a visible, inspectable product surface.
- Keep hero type restrained: `56px` to `72px` desktop, `40px` to `48px` mobile.
- Avoid decorative abstract art when product screenshots or realistic UI can carry the message.

### Docs

- Use light text on dark surfaces by default.
- Keep prose widths near `624px`.
- Use code blocks in the monospace stack with `14px` text and `24px` line height.
- Prefer precise navigation and clear section hierarchy over marketing visuals.

### Mobile

- Maintain the same dark palette.
- Respect minimum tap size `44px`.
- Prefer compact native lists, bottom sheets, and segmented controls.
- Pairing flows should make the QR/session state visually central.

### Desktop

- Use command menu, split panes, side panels, and keyboard-first interaction.
- Make app chrome quiet and predictable.
- Use subtle elevation for windows, dialogs, and panels.

## Common Patterns

- Dark sidebars with muted navigation and active indigo accents.
- Command palette centered over a dim overlay.
- Framed product previews with thin translucent borders and slight inner shadow.
- Compact buttons with icons and short labels.
- Status chips using muted semantic backgrounds.
- Tables and lists with row hover at `#191A1B`.
- Code/log panels with line numbers in `#62666D`.

## Design Principles

1. Product first: show the real Kloucy workflow before decorative branding.
2. Precision over decoration: rely on alignment, spacing, contrast, and type.
3. Dark by default: Kloucy should feel like focused development software.
4. Indigo with restraint: brand color marks focus and action, not every surface.
5. Compact and scannable: optimize for repeated use by builders.
6. Quiet confidence: avoid loud palettes, oversized cards, and ornamental effects.
7. Agent-native: make human and agent activity visible, traceable, and controllable.
8. Cross-surface consistency: web, docs, mobile, and desktop should share tokens and behavior.
