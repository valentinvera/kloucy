export interface MobileWorkspace {
  mobilePairingEnabled: boolean
}

export interface MobileWorkspaceList<T extends MobileWorkspace> {
  hidden: number
  visible: T[]
}

/** Workspaces shared with mobile, plus how many are not. */
export function mobileWorkspaces<T extends MobileWorkspace>(
  workspaces: T[]
): MobileWorkspaceList<T> {
  const visible = workspaces.filter(
    (workspace) => workspace.mobilePairingEnabled
  )

  return { hidden: workspaces.length - visible.length, visible }
}

export function hiddenNote(hidden: number): string | null {
  if (hidden <= 0) {
    return null
  }

  return hidden === 1
    ? "1 workspace is not shared with mobile."
    : `${hidden} workspaces are not shared with mobile.`
}
