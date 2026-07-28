import { expect, test } from "@playwright/test";

import { randomEmail, randomPassword, randomSuffix } from "./utils/random";
import { signIn, signOut, signUp } from "./utils/user";

test.use({ storageState: { cookies: [], origins: [] } });

test("updates profile details", async ({ page }) => {
  const email = randomEmail();
  const password = randomPassword();
  const name = `Profile ${randomSuffix()}`;

  await signUp(page, { email, password });
  await page.goto("/settings");
  await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();

  await page.locator("#full_name").fill(name);
  await page.getByRole("button", { name: "Save profile" }).click();
  await expect(page.getByText("Profile updated")).toBeVisible();

  await page.goto("/dashboard");
  await expect(page.getByText(name)).toBeVisible();
});

test("changes password and signs in with the new one", async ({ page }) => {
  const email = randomEmail();
  const password = randomPassword();
  const nextPassword = randomPassword();

  await signUp(page, { email, password });
  await page.goto("/settings");
  await page.locator("#current_password").fill(password);
  await page.locator("#new_password").fill(nextPassword);
  await page.locator("#confirm_password").fill(nextPassword);
  await page.getByRole("button", { name: "Update password" }).click();
  await expect(page.getByText(/Password updated/)).toBeVisible();

  await signOut(page);
  await signIn(page, email, nextPassword);
});
