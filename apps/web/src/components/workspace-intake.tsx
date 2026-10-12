import { Button } from "@kloucy/ui/components/button"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
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
  const [safeCommands, setSafeCommands] = useState("")
  const [ignoredPaths, setIgnoredPaths] = useState("")

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

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      completeIntake.mutate({
        id: workspaceId,
        ...(projectType ? { projectType } : {}),
        ...(safeCommands ? { safeCommands: splitList(safeCommands) } : {}),
        ...(ignoredPaths ? { ignoredPaths: splitList(ignoredPaths) } : {}),
      })
    },
    [completeIntake, ignoredPaths, projectType, safeCommands, workspaceId]
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
