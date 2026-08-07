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
    sans: '"Inter Variable", Inter, ui-sans-serif, system-ui, sans-serif',
  },
  fontWeight: {
    bold: 700,
    medium: 500,
    regular: 400,
    semibold: 600,
  },
  text: {
    caption: { fontSize: "0.75rem", lineHeight: "1rem" },
    display: { fontSize: "5.5rem", lineHeight: "5.75rem" },
    large: { fontSize: "1.125rem", lineHeight: "1.75rem" },
    medium: { fontSize: "1rem", lineHeight: "1.5rem" },
    small: { fontSize: "0.875rem", lineHeight: "1.25rem" },
    title: { fontSize: "3.5rem", lineHeight: "4rem" },
    xlarge: { fontSize: "1.5rem", lineHeight: "2rem" },
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

export { colors, effects, radius, semantic, spacing, typography }
