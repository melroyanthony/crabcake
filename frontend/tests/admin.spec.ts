import { expect, test } from "@playwright/test";

import { firstSuperuser } from "./config";
import { randomEmail, randomPassword, randomSuffix } from "./utils/random";

test("lists the first superuser and creates another user", async ({ page }) => {
  const email = randomEmail();
  const password = randomPassword();
  const name = `Admin ${randomSuffix()}`;

  await page.goto("/admin");
  await expect(page.getByRole("heading", { name: "Users" })).toBeVisible();
  await expect(page.getByRole("cell", { name: firstSuperuser })).toBeVisible();

  await page.getByRole("button", { name: "New user" }).click();
  await expect(page.getByRole("heading", { name: "New user" })).toBeVisible();
  const dialog = page.getByRole("dialog");
  await page.locator("#email").fill(email);
  await page.locator("#full_name").fill(name);
  await page.locator("#password").fill(password);
  await dialog.getByRole("button", { name: "Save" }).click();
  await expect(dialog).toBeHidden();
  await expect(page.getByRole("cell", { name: email })).toBeVisible();
});
