import { describe, expect, test } from "bun:test"
import {
  nativeMonoFontFamily,
  typography,
} from "../../packages/tokens/src/index"

describe("nativeMonoFontFamily", () => {
  test("uses the Android family name", () => {
    expect(nativeMonoFontFamily("android")).toBe("monospace")
  })

  test("uses Menlo on the Apple platforms", () => {
    expect(nativeMonoFontFamily("ios")).toBe("Menlo")
    expect(nativeMonoFontFamily("macos")).toBe("Menlo")
  })

  test("keeps the CSS stack on the web build", () => {
    expect(nativeMonoFontFamily("web")).toBe(typography.fontFamily.mono)
  })
})
