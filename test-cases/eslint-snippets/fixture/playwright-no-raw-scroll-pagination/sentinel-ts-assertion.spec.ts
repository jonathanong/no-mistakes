import type { Locator, Page } from "@playwright/test";

export async function scrollAssertedSentinel(page: Page) {
  const sentinel = page.getByTestId("bottom") as Locator;
  await sentinel.scrollIntoViewIfNeeded();
}
