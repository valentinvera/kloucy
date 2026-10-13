import { expect, type Page, test } from "@playwright/test"

const LANDING_HEADING = /run local agents/i
const SIGN_IN_PATH = /\/sign-in$/
const SIGN_UP_PATH = /\/sign-up$/

const INVALID_EMAIL = "nobody@example.com"
const INVALID_PASSWORD = "not-the-right-password"

async function signInWithInvalidCredentials(page: Page): Promise<boolean> {
  const post = page
    .waitForResponse(
      (response) => response.url().includes("/api/auth/sign-in"),
      { timeout: 2000 }
    )
    .catch(() => null)

  await page.getByLabel("Email").fill(INVALID_EMAIL)
  await page.getByLabel("Password").fill(INVALID_PASSWORD)
  await page.getByRole("button", { name: "Sign in" }).click()

  return (await post) !== null
}

test("loads the landing page", async ({ page }) => {
  await page.goto("/")

  await expect(
    page.getByRole("heading", { level: 1, name: LANDING_HEADING })
  ).toBeVisible()
  await expect(
    page.getByText("Agent orchestration for local projects")
  ).toBeVisible()
  await expect(page.getByText("Waiting for your approval.")).toBeVisible()

  await page.screenshot({
    fullPage: true,
    path: ".playwright-output/test-results/landing.png",
  })
})

test("links into the account entry points", async ({ page }) => {
  await page.goto("/")

  await page.getByRole("link", { name: "Create account" }).first().click()

  await expect(page).toHaveURL(SIGN_UP_PATH)
  await expect(
    page.getByRole("heading", { level: 1, name: "Create your account" })
  ).toBeVisible()
})

test("sends signed-out visitors to sign in", async ({ page }) => {
  await page.goto("/workspaces")

  await expect(page).toHaveURL(SIGN_IN_PATH)
  await expect(
    page.getByRole("heading", { level: 1, name: "Sign in" })
  ).toBeVisible()
})

test("reports a failed sign in", async ({ page }) => {
  await page.goto("/sign-in")

  // In dev the app hydrates after the load event, so a submit this early can
  // fall through to a native form post. Retrying once reaches the API.
  let submitted = await signInWithInvalidCredentials(page)

  if (!submitted) {
    submitted = await signInWithInvalidCredentials(page)
  }

  expect(submitted).toBe(true)

  await expect(page.getByRole("alert")).toBeVisible()
})
