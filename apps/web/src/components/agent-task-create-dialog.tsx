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

interface AgentTaskCreateDialogProps {
  workspaceId: string
}

export function AgentTaskCreateDialog({
  workspaceId,
}: AgentTaskCreateDialogProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const [open, setOpen] = useState(false)
  const [title, setTitle] = useState("")
  const [prompt, setPrompt] = useState("")
  const [error, setError] = useState<string | null>(null)

  const createTask = useMutation(
    trpc.agentTasks.create.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.agentTasks.list.queryFilter({ workspaceId })
        )
        setOpen(false)
        setTitle("")
        setPrompt("")
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

  const handleTitleChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setTitle(event.target.value)
    },
    []
  )

  const handlePromptChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setPrompt(event.target.value)
    },
    []
  )

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      setError(null)
      createTask.mutate(
        {
          prompt: prompt || undefined,
          title,
          workspaceId,
        },
        {
          onError: (mutationError) => {
            setError(mutationError.message)
          },
        }
      )
    },
    [createTask, prompt, title, workspaceId]
  )

  return (
    <>
      <Button onClick={handleOpen} size="sm">
        New task
      </Button>
      <Dialog onOpenChange={handleOpenChange} open={open}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New task</DialogTitle>
            <DialogDescription>
              Capture work before assigning it to an agent.
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
                <FieldLabel htmlFor="task-title">Title</FieldLabel>
                <Input
                  autoComplete="off"
                  id="task-title"
                  onChange={handleTitleChange}
                  required
                  value={title}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="task-prompt">Prompt</FieldLabel>
                <Input
                  autoComplete="off"
                  id="task-prompt"
                  onChange={handlePromptChange}
                  placeholder="What should the agent do?"
                  value={prompt}
                />
                <FieldDescription>
                  Optional. The instruction the agent runs.
                </FieldDescription>
              </Field>
            </FieldGroup>
            <DialogFooter className="mt-4">
              <Button
                disabled={createTask.isPending}
                onClick={handleCancel}
                variant="ghost"
              >
                Cancel
              </Button>
              <Button disabled={createTask.isPending} type="submit">
                {createTask.isPending ? "Creating…" : "Create task"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}
