import { describe, expect, test } from "bun:test"
import {
  hiddenNote,
  mobileWorkspaces,
} from "../../apps/mobile/lib/workspace-filter"

describe("mobileWorkspaces", () => {
  test("keeps only the workspaces shared with mobile", () => {
    const { hidden, visible } = mobileWorkspaces([
      { id: "w1", mobilePairingEnabled: true },
      { id: "w2", mobilePairingEnabled: false },
      { id: "w3", mobilePairingEnabled: true },
    ])

    expect(visible.map((workspace) => workspace.id)).toEqual(["w1", "w3"])
    expect(hidden).toBe(1)
  })

  test("reports nothing hidden when everything is shared", () => {
    const { hidden, visible } = mobileWorkspaces([
      { id: "w1", mobilePairingEnabled: true },
    ])

    expect(visible).toHaveLength(1)
    expect(hidden).toBe(0)
  })

  test("handles an empty list", () => {
    expect(mobileWorkspaces([])).toEqual({ hidden: 0, visible: [] })
  })
})

describe("hiddenNote", () => {
  test("says nothing when nothing is hidden", () => {
    expect(hiddenNote(0)).toBeNull()
  })

  test("counts one workspace", () => {
    expect(hiddenNote(1)).toBe("1 workspace is not shared with mobile.")
  })

  test("pluralizes the rest", () => {
    expect(hiddenNote(3)).toBe("3 workspaces are not shared with mobile.")
  })
})
