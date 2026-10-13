import { nativeMonoFontFamily } from "@kloucy/tokens"
import { Platform } from "react-native"

/**
 * System monospace for literal content: terminal output, commands, file paths
 * and IDs. The platform choice lives in `@kloucy/tokens`.
 */
export const monoFont = nativeMonoFontFamily(Platform.OS)
