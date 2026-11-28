import { expect, type Frame, type Locator as PwLocator, type Page as PwPage } from "@playwright/test";
import type { Page as RealPlaywrightPage } from "playwright";

interface Locator {
  first(): Locator;
}
interface Page {
  locator(selector: string): Locator;
  primitive(): string;
}

declare function helperFn(): void;

export async function discardedLocators(
  page: PwPage,
  frame: Frame,
  fakePage: Page,
  realPage: RealPlaywrightPage,
) {
  // Positive cases (discarded-locator)
  page.locator(".save");
  void page.getByTestId("submit");
  page.getByRole("button").first();
  frame.getByText("Hello");
  page.frameLocator("#embed").getByRole("link");
  page.frameLocator("#embed");
  const btn: PwLocator = page.getByTestId("ok");
  btn.filter({ hasText: "OK" });
  realPage.getByTestId("real");

  // Counterexamples (allowed)
  await expect(page.getByTestId("submit")).toBeVisible();
  await page.getByTestId("submit").click();
  const button = page.getByTestId("submit");
  fakePage.locator(".not-playwright");
  fakePage.primitive();
  helperFn();
  return button;
}
