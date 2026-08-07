import { cn as cnfast } from "cnfast"

export function cn(
  ...inputs: Parameters<typeof cnfast>
): ReturnType<typeof cnfast> {
  return cnfast(...inputs)
}
