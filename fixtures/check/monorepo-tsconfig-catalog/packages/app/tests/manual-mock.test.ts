import { test, vi } from "vitest";

vi.mock("@lib/lazy.ts", () => ({ lazy: true }));

test("matches an extension-bearing manual mock through the package alias", async () => {
  await import("@lib/lazy");
});
