import { expect, test } from "@playwright/test";

import { firstSuperuser, firstSuperuserPassword } from "./config";
import { randomPassword } from "./utils/random";
import { signIn, signOut } from "./utils/user";

test.use({ storageState: { cookies: [], origins: [] } });

test("sign-in form is ready", async ({ page }) => {
  await page.goto("/login");
  await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
  await expect(page.locator("#email")).toBeEditable();
  await expect(page.locator("#password")).toBeEditable();
  await expect(page.getByRole("button", { name: "Sign in" })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Forgot password?" }),
  ).toBeVisible();
});

test("signs in with the first superuser", async ({ page }) => {
  await signIn(page, firstSuperuser, firstSuperuserPassword);
  await expect(page.getByText(firstSuperuser)).toBeVisible();
});

test("rejects a wrong password", async ({ page }) => {
  await page.goto("/login");
  await page.locator("#email").fill(firstSuperuser);
  await page.locator("#password").fill(randomPassword());
  await page.getByRole("button", { name: "Sign in" }).click();
  await expect(page.getByText("not authenticated")).toBeVisible();
  await expect(page).toHaveURL(/\/login/);
});

test("signs out and blocks protected routes", async ({ page }) => {
  await signIn(page, firstSuperuser, firstSuperuserPassword);
  await signOut(page);
  await page.goto("/settings");
  await page.waitForURL(/\/login/);
});
