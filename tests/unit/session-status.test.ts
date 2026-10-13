import { describe, expect, test } from "bun:test"
import {
  sessionStatusClass,
  sessionStatusLabel,
} from "../../apps/web/src/lib/session-status"

describe("sessionStatusLabel", () => {
  test("labels known statuses", () => {
    expect(sessionStatusLabel("needs_setup")).toBe("Needs setup")
    expect(sessionStatusLabel("waiting")).toBe("Waiting")
  })

  test("passes unknown statuses through", () => {
    expect(sessionStatusLabel("paused")).toBe("paused")
  })
})

describe("sessionStatusClass", () => {
  test("uses the semantic token per status", () => {
    expect(sessionStatusClass("running")).toBe("bg-info text-info-foreground")
    expect(sessionStatusClass("waiting")).toBe(
      "bg-warning text-warning-foreground"
    )
    expect(sessionStatusClass("failed")).toBe(
      "bg-destructive text-destructive-foreground"
    )
    expect(sessionStatusClass("done")).toBe(
      "bg-success text-success-foreground"
    )
  })

  test("has no class for an unknown status", () => {
    expect(sessionStatusClass("paused")).toBe("")
  })
})
