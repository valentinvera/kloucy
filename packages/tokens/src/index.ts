const colors = {
  amber: {
    background: "#FA820022",
    foreground: "#FFCA16",
  },
  black: "#000000",
  blue: {
    background: "#0077FF3A",
    foreground: "#70B8FF",
  },
  gray: {
    background: "#16171AEB",
    foreground: "#FDFEFFA6",
  },
  green: {
    background: "#22FF991E",
    foreground: "#46FEA5D4",
  },
  red: {
    background: "#FF173F2D",
    foreground: "#FF9592",
  },
  white: "#FDFDFD",
} as const

const typography = {
  fontFamily: {
    mono: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    sans: '"Geist Variable", Geist, ui-sans-serif, system-ui, sans-serif',
  },
  fontWeight: {
    bold: 700,
    medium: 500,
    regular: 400,
    semibold: 600,
  },
  text: {
    action: { fontSize: "0.875rem", lineHeight: "1.25rem" },
    body: { fontSize: "1rem", lineHeight: "1.5rem" },
    caption: { fontSize: "0.75rem", lineHeight: "1.05rem" },
    compact: { fontSize: "0.6875rem", lineHeight: "0.9625rem" },
    control: { fontSize: "0.875rem", lineHeight: "1.25rem" },
    demoBody: { fontSize: "1rem", lineHeight: "1.5rem" },
    demoControl: { fontSize: "0.875rem", lineHeight: "1.25rem" },
    demoMetadata: { fontSize: "0.75rem", lineHeight: "1.125rem" },
    demoScore: { fontSize: "3.5rem", lineHeight: "4rem" },
    demoStat: { fontSize: "1.5rem", lineHeight: "2rem" },
    demoTitle: { fontSize: "1.25rem", lineHeight: "1.75rem" },
    display: { fontSize: "3.5rem", lineHeight: "3.875rem" },
    displayLg: { fontSize: "4rem", lineHeight: "4rem" },
    displayXl: { fontSize: "4.5rem", lineHeight: "4.5rem" },
    emphasis: { fontSize: "1.0625rem", lineHeight: "1.7rem" },
    footer: { fontSize: "0.875rem", lineHeight: "1.25rem" },
    hero: { fontSize: "2.375rem", lineHeight: "2.6125rem" },
    heroTitle: { fontSize: "2.875rem", lineHeight: "3.1875rem" },
    heroTitleXl: { fontSize: "6rem", lineHeight: "6rem" },
    large: { fontSize: "0.9375rem", lineHeight: "1.5rem" },
    medium: { fontSize: "0.875rem", lineHeight: "1.3125rem" },
    micro: { fontSize: "0.625rem", lineHeight: "0.9375rem" },
    nav: { fontSize: "0.875rem", lineHeight: "1.25rem" },
    paragraphMd: { fontSize: "1.125rem", lineHeight: "1.75rem" },
    paragraphXl: { fontSize: "1.25rem", lineHeight: "1.875rem" },
    section: { fontSize: "1.5rem", lineHeight: "1.995rem" },
    sectionLg: { fontSize: "2.5rem", lineHeight: "2.75rem" },
    small: { fontSize: "0.8125rem", lineHeight: "1.21875rem" },
    subtitle: { fontSize: "2rem", lineHeight: "2.25rem" },
    title: { fontSize: "3rem", lineHeight: "3rem" },
    xlarge: { fontSize: "1.25rem", lineHeight: "1.6625rem" },
  },
} as const

const radius = {
  "2xl": "1rem",
  "3xl": "1.5rem",
  "4xl": "2rem",
  lg: "0.5rem",
  md: "0.375rem",
  sm: "0.25rem",
  xl: "0.75rem",
  xs: "0.125rem",
} as const

const spacing = {
  unit: "0.25rem",
} as const

const effects = {
  blur: {
    glass: "25px",
  },
  shadow: {
    lg: "0 24px 72px -36px rgb(0 0 0 / 0.7)",
    md: "0 16px 48px -28px rgb(0 0 0 / 0.55)",
    sm: "0 8px 24px -16px rgb(0 0 0 / 0.4)",
    xs: "0 1px 2px 0 rgb(0 0 0 / 0.24)",
  },
} as const

const semantic = {
  accent: colors.blue.background,
  accentForeground: colors.blue.foreground,
  background: colors.black,
  border: "rgb(253 253 253 / 0.1)",
  card: colors.gray.background,
  cardForeground: colors.white,
  destructive: colors.red.background,
  destructiveForeground: colors.red.foreground,
  foreground: colors.white,
  info: colors.blue.background,
  infoForeground: colors.blue.foreground,
  input: "rgb(253 253 253 / 0.15)",
  muted: "#16171A99",
  mutedForeground: colors.gray.foreground,
  popover: "#111214",
  popoverForeground: colors.white,
  primary: colors.white,
  primaryForeground: colors.black,
  ring: colors.blue.foreground,
  secondary: colors.gray.background,
  secondaryForeground: colors.white,
  success: colors.green.background,
  successForeground: colors.green.foreground,
  warning: colors.amber.background,
  warningForeground: colors.amber.foreground,
} as const

export const kloucyTokens = {
  colors,
  effects,
  radius,
  semantic,
  spacing,
  typography,
} as const

export type KloucyTokens = typeof kloucyTokens

/**
 * Native surfaces take a single family name rather than a stack, so code,
 * terminal output and paths pick the platform's own monospace: Android accepts
 * `monospace`, iOS and macOS expose Menlo, and react-native-web keeps the CSS
 * stack from the tokens.
 */
export function nativeMonoFontFamily(platform: string): string {
  if (platform === "android") {
    return "monospace"
  }

  if (platform === "web") {
    return typography.fontFamily.mono
  }

  return "Menlo"
}

export { colors, effects, radius, semantic, spacing, typography }
