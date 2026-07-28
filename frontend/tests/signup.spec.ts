import { expect, test } from "@playwright/test";

import { randomEmail, randomPassword } from "./utils/random";
import { signUp } from "./utils/user";

test.use({ storageState: { cookies: [], origins: [] } });

test("creates an account and lands on the dashboard", async ({ page }) => {
  const email = randomEmail();
  const password = randomPassword();

  await signUp(page, { email, password, fullName: "E2E User" });
  await expect(page.getByText(/E2E User|Signed in as/)).toBeVisible();
});

test("rejects a password confirmation mismatch", async ({ page }) => {
  await page.goto("/signup");
  await page.locator("#email").fill(randomEmail());
  await page.locator("#password").fill(randomPassword());
  await page.locator("#confirm_password").fill(randomPassword());
  await page.getByRole("button", { name: "Create account" }).click();
  await expect(page.getByText("Passwords do not match")).toBeVisible();
});
