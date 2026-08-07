import { expect, test } from "@playwright/test"

test("loads the landing page", async ({ page }) => {
  await page.goto("/")

  await expect(page.getByText("Hello World")).toBeVisible()

  await page.screenshot({
    fullPage: true,
    path: ".playwright-output/test-results/landing.png",
  })
})

test("can interact with the landing page", async ({ page }) => {
  await page.goto("/")

  await page.getByText("Hello World").click()
  await page.keyboard.press("Tab")

  await expect(page).toHaveURL("/")
  await page.screenshot({
    fullPage: true,
    path: ".playwright-output/test-results/landing-interaction.png",
  })
})
