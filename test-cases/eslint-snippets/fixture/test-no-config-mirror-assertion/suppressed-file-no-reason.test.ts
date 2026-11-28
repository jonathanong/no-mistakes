// no-mistakes-disable-file test-no-config-mirror-assertion
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("is not suppressed when file directive lacks a reason", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
});
