import { readFileSync } from "node:fs";
import { expect } from "vitest";

export function checkWorkflow() {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
}
