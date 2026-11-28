import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

const EXPECTED_LINE = "persist-credentials: false";

it("flags all literal sink matchers and their .not forms", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toBeDefined();
  expect(text).toContain(EXPECTED_LINE);
  expect(text).not.toContain("pull_request_target");
  expect(text).toMatch(/timeout-minutes:\s*15/);
  expect(text).not.toMatch(/write-all/);
  expect(text).toMatch(new RegExp("persist-credentials", "u"));
});

it("flags equality sinks on parsed JSON values", () => {
  const pkg = JSON.parse(readFileSync("package.json", "utf8"));
  expect(pkg.private).toBe(true);
  expect(pkg.private).not.toBe(false);
  expect(pkg.keywords).toEqual(["cli", "ast"]);
  expect(pkg.keywords).not.toEqual(["other"]);
  expect(pkg.engines).toStrictEqual({ node: ">=20" });
  expect(pkg.engines).not.toStrictEqual({ node: ">=18" });
});
