import { expect, test } from "@playwright/test";

import {
  findLastEmail,
  recipientMatches,
  resetPasswordUrl,
} from "./utils/mailcatcher";
import { randomEmail, randomPassword } from "./utils/random";
import { signIn, signOut, signUp } from "./utils/user";

test.use({ storageState: { cookies: [], origins: [] } });

test("recovers a password from the emailed link", async ({ page, request }) => {
  const email = randomEmail();
  const password = randomPassword();
  const nextPassword = randomPassword();

  await signUp(page, { email, password, fullName: "Reset User" });
  await signOut(page);

  await page.goto("/recover-password");
  await page.locator("#email").fill(email);
  await page.getByRole("button", { name: "Send reset link" }).click();
  await expect(
    page.getByText("If that email exists, a reset link is on its way."),
  ).toBeVisible();

  const message = await findLastEmail({
    request,
    filter: (candidate) => recipientMatches(candidate, email),
  });
  const url = await resetPasswordUrl(request, message.id);

  await page.goto(url);
  await expect(
    page.getByRole("heading", { name: "Choose a new password" }),
  ).toBeVisible();
  await page.locator("#new_password").fill(nextPassword);
  await page.locator("#confirm_password").fill(nextPassword);
  await page.getByRole("button", { name: "Update password" }).click();
  await page.waitForURL("/login");

  await signIn(page, email, nextPassword);
});

test("rejects an invalid reset token", async ({ page }) => {
  const password = randomPassword();
  await page.goto("/reset-password?token=not-a-real-token");
  await page.locator("#new_password").fill(password);
  await page.locator("#confirm_password").fill(password);
  await page.getByRole("button", { name: "Update password" }).click();
  await expect(
    page.getByText("that reset link is invalid or has expired"),
  ).toBeVisible();
});
