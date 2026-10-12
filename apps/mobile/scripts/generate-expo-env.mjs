import { writeFile } from "node:fs/promises"
import { join } from "node:path"

// Expo writes expo-env.d.ts when its dev server starts. CI and fresh clones
// never start it, so we generate the same file before type checking.
const template = `/// <reference types="expo/types" />

// NOTE: This file should not be edited and should be in your git ignore`

const projectRoot = join(import.meta.dirname, "..")

await writeFile(join(projectRoot, "expo-env.d.ts"), template)
