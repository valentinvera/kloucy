import { describe, expect, test } from "bun:test"
import {
  formatDate,
  stageClass,
  stageLabel,
} from "../../apps/web/src/lib/workspace-display"

describe("stageLabel", () => {
  test("labels known stages", () => {
    expect(stageLabel("setup")).toBe("Setup")
    expect(stageLabel("ready")).toBe("Ready")
  })

  test("passes unknown stages through", () => {
    expect(stageLabel("whatever")).toBe("whatever")
  })
})

describe("stageClass", () => {
  test("uses the semantic token per stage", () => {
    expect(stageClass("ready")).toBe(
      "border-transparent bg-info text-info-foreground"
    )
    expect(stageClass("paused")).toBe(
      "border-transparent bg-warning text-warning-foreground"
    )
  })

  test("has no class for an unknown stage", () => {
    expect(stageClass("whatever")).toBe("")
  })
})

describe("formatDate", () => {
  test("formats a date value", () => {
    expect(formatDate(new Date("2026-01-15T12:00:00Z"))).toBe("Jan 15, 2026")
  })

  test("accepts an ISO string", () => {
    expect(formatDate("2026-01-15T12:00:00Z")).toBe("Jan 15, 2026")
  })
})
