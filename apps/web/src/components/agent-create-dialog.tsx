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
import { type ChangeEvent, type FormEvent, useCallback, useState } from "react"
import { useTRPC } from "@/utils/trpc"

export function AgentCreateDialog() {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const [open, setOpen] = useState(false)
  const [name, setName] = useState("")
  const [provider, setProvider] = useState("")
  const [executablePath, setExecutablePath] = useState("")
  const [error, setError] = useState<string | null>(null)

  const createAgent = useMutation(
    trpc.agents.create.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(trpc.agents.list.queryFilter())
        setOpen(false)
        setName("")
        setProvider("")
        setExecutablePath("")
      },
    })
  )

  const handleOpen = useCallback(() => {
    setOpen(true)
  }, [])

  const handleOpenChange = useCallback((next: boolean) => {
    setOpen(next)
    if (!next) {
      setError(null)
    }
  }, [])

  const handleCancel = useCallback(() => {
    setOpen(false)
    setError(null)
  }, [])

  const handleNameChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setName(event.target.value)
    },
    []
  )

  const handleProviderChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setProvider(event.target.value)
    },
    []
  )

  const handlePathChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setExecutablePath(event.target.value)
    },
    []
  )

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      setError(null)
      createAgent.mutate(
        {
          executablePath: executablePath || undefined,
          name,
          provider,
        },
        {
          onError: (mutationError) => {
            setError(mutationError.message)
          },
        }
      )
    },
    [createAgent, executablePath, name, provider]
  )

  return (
    <>
      <Button onClick={handleOpen}>New agent</Button>
      <Dialog onOpenChange={handleOpenChange} open={open}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New agent</DialogTitle>
            <DialogDescription>
              Register a local agent Kloucy can run tasks with.
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
                <FieldLabel htmlFor="agent-name">Name</FieldLabel>
                <Input
                  autoComplete="off"
                  id="agent-name"
                  onChange={handleNameChange}
                  required
                  value={name}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="agent-provider">Provider</FieldLabel>
                <Input
                  autoComplete="off"
                  id="agent-provider"
                  onChange={handleProviderChange}
                  placeholder="claude-code"
                  required
                  value={provider}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="agent-path">Executable path</FieldLabel>
                <Input
                  autoComplete="off"
                  className="font-mono"
                  id="agent-path"
                  onChange={handlePathChange}
                  placeholder="/usr/local/bin/agent"
                  value={executablePath}
                />
                <FieldDescription>
                  Optional. The command Kloucy runs for this agent.
                </FieldDescription>
              </Field>
            </FieldGroup>
            <DialogFooter className="mt-4">
              <Button
                disabled={createAgent.isPending}
                onClick={handleCancel}
                variant="ghost"
              >
                Cancel
              </Button>
              <Button disabled={createAgent.isPending} type="submit">
                {createAgent.isPending ? "Creating…" : "Create agent"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}
