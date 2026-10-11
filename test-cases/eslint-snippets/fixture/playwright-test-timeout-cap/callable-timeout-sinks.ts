import { test, defineConfig } from "@playwright/test";
// Function projection must not become a numeric timeout or a known slow deadline.
test.setTimeout(() => 1);
test.setTimeout(() => 60_000);
defineConfig({ timeout: () => 1 });
test("callable slow deadline", () => {
  test.setTimeout(() => 1);
  test.slow();
});
test.setTimeout(1);
test.setTimeout(60_000);
