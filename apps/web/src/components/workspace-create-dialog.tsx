import { Alert, AlertDescription } from "@kloucy/ui/components/alert"
import { Button } from "@kloucy/ui/components/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@kloucy/ui/components/dialog"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { type ChangeEvent, type FormEvent, useCallback, useState } from "react"
import { useTRPC } from "@/utils/trpc"

export function WorkspaceCreateDialog() {
  const trpc = useTRPC()
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [localPath, setLocalPath] = useState("")
  const [error, setError] = useState<string | null>(null)

  const createWorkspace = useMutation(
    trpc.workspaces.create.mutationOptions({
      onSuccess: async (workspace) => {
        await queryClient.invalidateQueries(trpc.workspaces.list.queryFilter())
        setOpen(false)
        setName("")
        setLocalPath("")
        await navigate({
          params: { workspaceId: workspace.id },
          to: "/workspaces/$workspaceId",
        })
      },
    })
  )

  const handleOpen = useCallback(() => {
    setOpen(true)
  }, [])

  const handleCancel = useCallback(() => {
    setOpen(false)
    setError(null)
  }, [])

  const handleOpenChange = useCallback((next: boolean) => {
    setOpen(next)
    if (!next) {
      setError(null)
    }
  }, [])

  const handleNameChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setName(event.target.value)
    },
    []
  )

  const handlePathChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setLocalPath(event.target.value)
    },
    []
  )

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      setError(null)
      createWorkspace.mutate(
        { localPath, name },
        {
          onError: (mutationError) => {
            setError(mutationError.message)
          },
        }
      )
    },
    [createWorkspace, localPath, name]
  )

  return (
    <>
      <Button onClick={handleOpen}>New workspace</Button>
      <Dialog onOpenChange={handleOpenChange} open={open}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New workspace</DialogTitle>
            <DialogDescription>
              Point Kloucy at a local project to start supervising agents.
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSubmit}>
            <FieldGroup>
              {error ? (
                <Alert variant="destructive">
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              ) : null}
              <Field>
                <FieldLabel htmlFor="workspace-name">Name</FieldLabel>
                <Input
                  autoComplete="off"
                  id="workspace-name"
                  onChange={handleNameChange}
                  required
                  value={name}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="workspace-path">Local path</FieldLabel>
                <Input
                  autoComplete="off"
                  className="font-mono"
                  id="workspace-path"
                  onChange={handlePathChange}
                  placeholder="/Users/you/project"
                  required
                  value={localPath}
                />
                <FieldDescription>
                  Absolute path to the project on this machine.
                </FieldDescription>
              </Field>
            </FieldGroup>
            <DialogFooter className="mt-4">
              <Button
                disabled={createWorkspace.isPending}
                onClick={handleCancel}
                variant="ghost"
              >
                Cancel
              </Button>
              <Button disabled={createWorkspace.isPending} type="submit">
                {createWorkspace.isPending ? "Creating…" : "Create workspace"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}
