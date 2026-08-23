import { test, vi } from "vitest";

// The extension-bearing mock and extensionless import resolve through the
// symlinked root to the same lexical graph node.
vi.mock("@linked/value.ts", () => ({ value: "mocked" }));

test("matches a manual mock through a symlinked alias", async () => {
  await import("@linked/value");
});
