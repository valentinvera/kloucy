import "reflect-metadata"
import { env, getCorsOrigins } from "@kloucy/env/api"
import { NestFactory } from "@nestjs/core"
import { AppModule } from "./app.module"

async function bootstrap(): Promise<void> {
  const app = await NestFactory.create(AppModule)

  app.setGlobalPrefix("api")

  app.enableCors({
    allowedHeaders: ["Content-Type", "Authorization"],
    credentials: true,
    methods: ["GET", "POST", "OPTIONS"],
    origin: getCorsOrigins(),
  })

  await app.listen(env.PORT, env.HOST)
}

await bootstrap()
