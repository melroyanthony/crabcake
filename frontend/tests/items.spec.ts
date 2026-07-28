import { expect, test } from "@playwright/test";

import { randomTitle } from "./utils/random";

test("creates, edits and deletes an item", async ({ page }) => {
  const title = randomTitle();
  const updated = `${title} edited`;

  await page.goto("/items");
  await expect(page.getByRole("heading", { name: "Items" })).toBeVisible();

  await page.getByRole("button", { name: "New item" }).click();
  await expect(page.getByRole("heading", { name: "New item" })).toBeVisible();
  const dialog = page.getByRole("dialog");
  await page.locator("#title").fill(title);
  await page.locator("#description").fill("Created by Playwright");
  await dialog.getByRole("button", { name: "Save" }).click();
  await expect(dialog).toBeHidden();
  await expect(page.getByRole("cell", { name: title })).toBeVisible();

  const row = page.getByRole("row").filter({ hasText: title });
  await row.getByRole("button", { name: "Edit" }).click();
  await expect(page.getByRole("heading", { name: "Edit item" })).toBeVisible();
  await page.locator("#title").fill(updated);
  await page.getByRole("dialog").getByRole("button", { name: "Save" }).click();
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(page.getByRole("cell", { name: updated })).toBeVisible();

  page.once("dialog", (dialog) => dialog.accept());
  await page
    .getByRole("row")
    .filter({ hasText: updated })
    .getByRole("button", { name: "Delete" })
    .click();
  await expect(page.getByRole("cell", { name: updated })).toHaveCount(0);
});
