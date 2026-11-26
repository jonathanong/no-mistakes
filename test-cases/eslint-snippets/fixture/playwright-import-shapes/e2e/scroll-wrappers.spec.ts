import { test } from "@playwright/test";

declare const page: {
  waitForResponse(predicate: (response: { url(): string }) => boolean): Promise<unknown>;
  getByTestId(testId: string): { scrollIntoViewIfNeeded(): Promise<void> };
};

page.waitForResponse((response) => response.url().includes("?cursor="));
window.scrollTo(0, 100);
(window as Window).scrollTo(0, 100);
page.getByTestId("feed-sentinel" as const).scrollIntoViewIfNeeded();
(page.getByTestId("feed-sentinel").scrollIntoViewIfNeeded as any)();

void test;
