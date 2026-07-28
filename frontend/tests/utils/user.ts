import { expect, type Page } from "@playwright/test";

export async function signOut(page: Page) {
  await page.getByRole("button", { name: "Sign out", exact: true }).click();
  await page.waitForURL("/login");
}

export async function signIn(page: Page, email: string, password: string) {
  await page.goto("/login");
  await page.locator("#email").fill(email);
  await page.locator("#password").fill(password);
  await page.getByRole("button", { name: "Sign in" }).click();
  await page.waitForURL("/dashboard");
  await expect(page.getByRole("heading", { name: "Dashboard" })).toBeVisible();
}

export async function signUp(
  page: Page,
  {
    email,
    password,
    fullName,
  }: { email: string; password: string; fullName?: string },
) {
  await page.goto("/signup");
  await page.locator("#email").fill(email);
  if (fullName) {
    await page.locator("#full_name").fill(fullName);
  }
  await page.locator("#password").fill(password);
  await page.locator("#confirm_password").fill(password);
  await page.getByRole("button", { name: "Create account" }).click();
  await page.waitForURL("/dashboard");
}
