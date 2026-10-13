import { Button } from "@kloucy/ui/components/button"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
import { Label } from "@kloucy/ui/components/label"
import { Switch } from "@kloucy/ui/components/switch"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { type ChangeEvent, type FormEvent, useCallback, useState } from "react"
import { useTRPC } from "@/utils/trpc"

interface WorkspaceIntakeProps {
  workspaceId: string
}

function splitList(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0)
}

export function WorkspaceIntake({ workspaceId }: WorkspaceIntakeProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const [projectType, setProjectType] = useState("")
  const [packageManager, setPackageManager] = useState("")
  const [runtime, setRuntime] = useState("")
  const [safeCommands, setSafeCommands] = useState("")
  const [ignoredPaths, setIgnoredPaths] = useState("")
  const [mobilePairing, setMobilePairing] = useState(false)
  const [guardCommands, setGuardCommands] = useState(false)

  const completeIntake = useMutation(
    trpc.workspaces.completeIntake.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.workspaces.get.queryFilter({ id: workspaceId })
        )
      },
    })
  )

  const handleProjectTypeChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setProjectType(event.target.value)
    },
    []
  )

  const handlePackageManagerChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setPackageManager(event.target.value)
    },
    []
  )

  const handleRuntimeChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setRuntime(event.target.value)
    },
    []
  )

  const handleSafeCommandsChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setSafeCommands(event.target.value)
    },
    []
  )

  const handleIgnoredPathsChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setIgnoredPaths(event.target.value)
    },
    []
  )

  const handleMobilePairingChange = useCallback((checked: boolean) => {
    setMobilePairing(checked)
  }, [])

  const handleGuardCommandsChange = useCallback((checked: boolean) => {
    setGuardCommands(checked)
  }, [])

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      completeIntake.mutate({
        id: workspaceId,
        mobilePairingEnabled: mobilePairing,
        safetyMode: guardCommands ? "guarded" : "open",
        ...(projectType ? { projectType } : {}),
        ...(packageManager ? { packageManager } : {}),
        ...(runtime ? { runtime } : {}),
        ...(safeCommands ? { safeCommands: splitList(safeCommands) } : {}),
        ...(ignoredPaths ? { ignoredPaths: splitList(ignoredPaths) } : {}),
      })
    },
    [
      completeIntake,
      guardCommands,
      ignoredPaths,
      mobilePairing,
      packageManager,
      projectType,
      runtime,
      safeCommands,
      workspaceId,
    ]
  )

  return (
    <div className="mt-10">
      <h2 className="font-medium text-medium">Workspace intake</h2>
      <form className="mt-4" onSubmit={handleSubmit}>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="intake-project-type">Project type</FieldLabel>
            <Input
              autoComplete="off"
              id="intake-project-type"
              onChange={handleProjectTypeChange}
              placeholder="rust, node, …"
              value={projectType}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="intake-package-manager">
              Package manager
            </FieldLabel>
            <Input
              autoComplete="off"
              id="intake-package-manager"
              onChange={handlePackageManagerChange}
              placeholder="bun, cargo, …"
              value={packageManager}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="intake-runtime">Runtime</FieldLabel>
            <Input
              autoComplete="off"
              id="intake-runtime"
              onChange={handleRuntimeChange}
              placeholder="bun, node, rust, …"
              value={runtime}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="intake-safe-commands">
              Safe commands
            </FieldLabel>
            <Input
              autoComplete="off"
              className="font-mono"
              id="intake-safe-commands"
              onChange={handleSafeCommandsChange}
              placeholder="cargo test, bun test"
              value={safeCommands}
            />
            <FieldDescription>Comma separated.</FieldDescription>
          </Field>
          <Field>
            <FieldLabel htmlFor="intake-ignored-paths">
              Ignored paths
            </FieldLabel>
            <Input
              autoComplete="off"
              className="font-mono"
              id="intake-ignored-paths"
              onChange={handleIgnoredPathsChange}
              placeholder=".env, secrets/"
              value={ignoredPaths}
            />
          </Field>
          <Field>
            <FieldTitle>Mobile supervision</FieldTitle>
            <div className="flex items-center gap-2">
              <Switch
                checked={mobilePairing}
                id="intake-mobile-pairing"
                onCheckedChange={handleMobilePairingChange}
              />
              <Label htmlFor="intake-mobile-pairing">
                {mobilePairing ? "Shared with mobile" : "Desktop only"}
              </Label>
            </div>
            <FieldDescription>
              Shared workspaces appear in the mobile app.
            </FieldDescription>
          </Field>
          <Field>
            <FieldTitle>Safety mode</FieldTitle>
            <div className="flex items-center gap-2">
              <Switch
                checked={guardCommands}
                id="intake-safety-mode"
                onCheckedChange={handleGuardCommandsChange}
              />
              <Label htmlFor="intake-safety-mode">
                {guardCommands
                  ? "Guarded: safe commands only"
                  : "Open: any command"}
              </Label>
            </div>
            <FieldDescription>
              A guarded workspace only starts a command listed in its safe
              commands.
            </FieldDescription>
          </Field>
        </FieldGroup>
        <Button
          className="mt-4"
          disabled={completeIntake.isPending}
          type="submit"
        >
          {completeIntake.isPending ? "Saving…" : "Complete intake"}
        </Button>
      </form>
    </div>
  )
}
