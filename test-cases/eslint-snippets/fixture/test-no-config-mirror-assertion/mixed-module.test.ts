import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import { validatePolicy } from "./validator";

const ci = readFileSync(".github/workflows/ci.yml", "utf8");
const pkg = JSON.parse(readFileSync("package.json", "utf8"));

it("allows cross-file test using module-level read", () => {
  expect(ci).toContain(pkg.engines.node);
});

it("flags mirror-only test using the same module-level read", () => {
  expect(ci).toContain("timeout-minutes: 15");
});

it("allows test passing module-level read to code under test", () => {
  expect(validatePolicy(ci)).toBe(true);
});
