import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("reads from fixtures and testdata", () => {
  expect(readFileSync("fixtures/ci.yml", "utf8")).toContain("persist-credentials: false");
  expect(readFileSync("testdata/ci.yml", "utf8")).toContain("persist-credentials: false");
});
