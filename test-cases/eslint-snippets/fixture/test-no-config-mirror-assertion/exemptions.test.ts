import { readFileSync } from "node:fs";
import { expect, it, vi } from "vitest";

vi.mock("node:fs", () => ({
  readFileSync: vi.fn(() => "persist-credentials: false"),
}));

it("ignores files that mock node:fs", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
});
