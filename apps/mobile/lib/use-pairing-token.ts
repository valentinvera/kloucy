import { parse, useURL } from "expo-linking"

export function usePairingToken(): string | null {
  const url = useURL()

  if (!url) {
    return null
  }

  const { queryParams } = parse(url)
  const token = queryParams?.token

  return typeof token === "string" && token.length > 0 ? token : null
}
