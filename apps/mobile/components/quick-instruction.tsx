import { Button, Input, Label, Spinner, TextField } from "heroui-native"
import { useCallback, useState } from "react"
import { Text, View } from "react-native"

export interface InstructionSession {
  id: string
  prompt: string | null
  status: string
}

interface SessionTargetProps {
  isSelected: boolean
  onSelect: (id: string) => void
  session: InstructionSession
}

function SessionTarget({ isSelected, onSelect, session }: SessionTargetProps) {
  const handleSelect = useCallback(
    () => onSelect(session.id),
    [onSelect, session.id]
  )

  return (
    <Button
      onPress={handleSelect}
      size="sm"
      variant={isSelected ? "primary" : "secondary"}
    >
      <Button.Label>{session.prompt ?? "Untitled session"}</Button.Label>
    </Button>
  )
}

interface QuickInstructionProps {
  isSending: boolean
  onSend: (sessionId: string, instruction: string) => void
  sessions: InstructionSession[]
}

export function QuickInstruction({
  isSending,
  onSend,
  sessions,
}: QuickInstructionProps) {
  const [targetId, setTargetId] = useState<string | null>(null)
  const [instruction, setInstruction] = useState("")

  const handleSend = useCallback(() => {
    const trimmed = instruction.trim()

    if (!(targetId && trimmed)) {
      return
    }

    onSend(targetId, trimmed)
    setInstruction("")
  }, [instruction, onSend, targetId])

  const canSend = Boolean(targetId) && instruction.trim().length > 0

  return (
    <View className="gap-4">
      <Text className="font-geist-semibold text-foreground text-lg">
        Quick instruction
      </Text>

      {sessions.length === 0 ? (
        <Text className="font-geist text-muted text-sm">
          Start a session on desktop to send it instructions.
        </Text>
      ) : (
        <View className="gap-3">
          <View className="flex-row flex-wrap gap-2">
            {sessions.map((session) => (
              <SessionTarget
                isSelected={session.id === targetId}
                key={session.id}
                onSelect={setTargetId}
                session={session}
              />
            ))}
          </View>

          <TextField>
            <Label>Instruction</Label>
            <Input
              onChangeText={setInstruction}
              placeholder="Tell the agent what to do next"
              value={instruction}
            />
          </TextField>

          <Button
            isDisabled={!(canSend && !isSending)}
            onPress={handleSend}
            variant="primary"
          >
            {isSending ? (
              <Spinner />
            ) : (
              <Button.Label>Send instruction</Button.Label>
            )}
          </Button>
        </View>
      )}
    </View>
  )
}
