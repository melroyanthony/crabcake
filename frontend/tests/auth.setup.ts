import { expect, test as setup } from "@playwright/test";

import { firstSuperuser, firstSuperuserPassword } from "./config";

const authFile = "playwright/.auth/user.json";

setup("authenticate as first superuser", async ({ page }) => {
  await page.goto("/login");
  await page.locator("#email").fill(firstSuperuser);
  await page.locator("#password").fill(firstSuperuserPassword);
  await page.getByRole("button", { name: "Sign in" }).click();
  await page.waitForURL("/dashboard");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
  await expect(page.getByText("· superuser")).toBeVisible();
  await page.context().storageState({ path: authFile });
});
