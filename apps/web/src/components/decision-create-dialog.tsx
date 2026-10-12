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
import { Field, FieldGroup, FieldLabel } from "@kloucy/ui/components/field"
import { Input } from "@kloucy/ui/components/input"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { type ChangeEvent, type FormEvent, useCallback, useState } from "react"
import { useTRPC } from "@/utils/trpc"

interface DecisionCreateDialogProps {
  workspaceId: string
}

export function DecisionCreateDialog({
  workspaceId,
}: DecisionCreateDialogProps) {
  const trpc = useTRPC()
  const queryClient = useQueryClient()

  const [open, setOpen] = useState(false)
  const [title, setTitle] = useState("")
  const [summary, setSummary] = useState("")
  const [error, setError] = useState<string | null>(null)

  const createDecision = useMutation(
    trpc.decisions.create.mutationOptions({
      onSuccess: async () => {
        await queryClient.invalidateQueries(
          trpc.decisions.list.queryFilter({ workspaceId })
        )
        setOpen(false)
        setTitle("")
        setSummary("")
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

  const handleSummaryChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setSummary(event.target.value)
    },
    []
  )

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      setError(null)
      createDecision.mutate(
        {
          summary: summary || undefined,
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
    [createDecision, summary, title, workspaceId]
  )

  return (
    <>
      <Button onClick={handleOpen} size="sm" variant="secondary">
        Add decision
      </Button>
      <Dialog onOpenChange={handleOpenChange} open={open}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add decision</DialogTitle>
            <DialogDescription>
              Record a control decision so its reason is not lost.
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
                <FieldLabel htmlFor="decision-title">Title</FieldLabel>
                <Input
                  autoComplete="off"
                  id="decision-title"
                  onChange={handleTitleChange}
                  required
                  value={title}
                />
              </Field>
              <Field>
                <FieldLabel htmlFor="decision-summary">Summary</FieldLabel>
                <Input
                  autoComplete="off"
                  id="decision-summary"
                  onChange={handleSummaryChange}
                  placeholder="What changed and why"
                  value={summary}
                />
              </Field>
            </FieldGroup>
            <DialogFooter className="mt-4">
              <Button
                disabled={createDecision.isPending}
                onClick={handleCancel}
                variant="ghost"
              >
                Cancel
              </Button>
              <Button disabled={createDecision.isPending} type="submit">
                {createDecision.isPending ? "Saving…" : "Add decision"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  )
}
