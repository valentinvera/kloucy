import dotenv from "dotenv"
import { type Config, defineConfig } from "drizzle-kit"

dotenv.config({
  path: "../../apps/api/.env",
})

export default defineConfig({
  dbCredentials: {
    url: process.env.DATABASE_URL || "",
  },
  dialect: "postgresql",
  out: "./drizzle",
  schema: "./src/schema/*.ts",
} satisfies Config)
