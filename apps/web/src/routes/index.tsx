import { Button } from "@kloucy/ui/components/button"
import { createFileRoute, Link } from "@tanstack/react-router"
import { LandingLocal } from "@/components/landing-local"
import { LandingLoop } from "@/components/landing-loop"
import { LandingPairing } from "@/components/landing-pairing"
import { LandingSessionBoard } from "@/components/landing-session-board"

export const Route = createFileRoute("/")({
  component: HomeComponent,
})

const REPOSITORY_URL = "https://github.com/valentinvera/kloucy"

function LandingHeader() {
  return (
    <header className="border-border/60 border-b">
      <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-6 py-4">
        <Link className="font-logo text-section" to="/">
          Kloucy
        </Link>
        <nav className="flex items-center gap-2">
          <Button render={<Link to="/sign-in" />} size="sm" variant="ghost">
            Sign in
          </Button>
          <Button render={<Link to="/sign-up" />} size="sm">
            Create account
          </Button>
        </nav>
      </div>
    </header>
  )
}

function LandingFooter() {
  return (
    <footer className="border-border/60 border-t">
      <div className="mx-auto flex max-w-6xl flex-col gap-4 px-6 py-10 sm:flex-row sm:items-center sm:justify-between">
        <span className="font-logo text-medium">Kloucy</span>
        <div className="flex flex-wrap items-center gap-4 text-caption text-muted-foreground">
          <a
            className="hover:text-foreground"
            href={REPOSITORY_URL}
            rel="noopener noreferrer"
            target="_blank"
          >
            GitHub
          </a>
          <Link className="hover:text-foreground" to="/sign-in">
            Sign in
          </Link>
          <Link className="hover:text-foreground" to="/sign-up">
            Create account
          </Link>
        </div>
      </div>
    </footer>
  )
}

function HomeComponent() {
  return (
    <div className="flex min-h-svh flex-col">
      <LandingHeader />

      <main className="flex-1">
        <section className="py-16 lg:py-24">
          <div className="mx-auto flex min-w-0 max-w-6xl flex-col gap-8 px-6">
            <span className="text-caption text-muted-foreground">
              Agent orchestration for local projects
            </span>
            <h1 className="max-w-[20ch] text-balance font-medium text-hero md:text-display-lg lg:text-hero-title-xl">
              Run local agents. See what changed. Stay in control.
            </h1>
            <p className="max-w-[56ch] text-body text-muted-foreground md:text-paragraph-md lg:text-paragraph-xl">
              Kloucy turns scattered local-agent work into sessions you can
              watch, review, and stop. Agents run on your machine, in your
              project, and every run leaves a record.
            </p>
            <div className="flex flex-wrap items-center gap-3">
              <Button render={<Link to="/sign-up" />} size="lg">
                Create account
              </Button>
              <Button
                render={<Link to="/sign-in" />}
                size="lg"
                variant="outline"
              >
                Sign in
              </Button>
            </div>
          </div>
          <div className="mx-auto mt-12 max-w-6xl px-6 lg:mt-16">
            <LandingSessionBoard />
          </div>
        </section>

        <section className="border-border/60 border-t">
          <div className="mx-auto flex max-w-6xl flex-col gap-10 px-6 py-16 lg:py-20">
            <div className="flex flex-col gap-3">
              <span className="text-caption text-muted-foreground">
                The loop
              </span>
              <h2 className="max-w-[24ch] text-balance font-medium text-subtitle md:text-section-lg lg:text-title">
                Capture, run, review.
              </h2>
              <p className="max-w-[56ch] text-body text-muted-foreground md:text-paragraph-md">
                Every session starts as something you wrote down and ends as
                something you approved.
              </p>
            </div>
            <LandingLoop />
          </div>
        </section>

        <section className="border-border/60 border-t">
          <div className="mx-auto flex max-w-6xl flex-col gap-10 px-6 py-16 lg:py-20">
            <div className="flex flex-col gap-3">
              <span className="text-caption text-muted-foreground">
                Local by design
              </span>
              <h2 className="max-w-[24ch] text-balance font-medium text-subtitle md:text-section-lg lg:text-title">
                Your machine, your data.
              </h2>
              <p className="max-w-[56ch] text-body text-muted-foreground md:text-paragraph-md">
                Kloucy runs next to your code and asks before it acts.
              </p>
            </div>
            <LandingLocal />
          </div>
        </section>

        <section className="border-border/60 border-t">
          <div className="mx-auto flex max-w-6xl flex-col gap-10 px-6 py-16 lg:py-20">
            <div className="flex flex-col gap-3">
              <span className="text-caption text-muted-foreground">
                Away from the desk
              </span>
              <h2 className="max-w-[24ch] text-balance font-medium text-subtitle md:text-section-lg lg:text-title">
                Keep control from your phone.
              </h2>
            </div>
            <LandingPairing />
          </div>
        </section>

        <section className="border-border/60 border-t">
          <div className="mx-auto flex max-w-6xl flex-col gap-6 px-6 py-16 lg:py-20">
            <span className="text-caption text-muted-foreground">
              Get started
            </span>
            <h2 className="max-w-[22ch] text-balance font-medium text-section-lg md:text-title lg:text-display">
              See what your agents are doing.
            </h2>
            <div className="flex flex-wrap items-center gap-3">
              <Button render={<Link to="/sign-up" />} size="lg">
                Create account
              </Button>
              <Button
                render={<Link to="/sign-in" />}
                size="lg"
                variant="outline"
              >
                Sign in
              </Button>
            </div>
            <p className="max-w-[56ch] text-body text-muted-foreground">
              The desktop app connects to your own API. Setup steps live in the
              repository docs.
            </p>
          </div>
        </section>
      </main>

      <LandingFooter />
    </div>
  )
}
