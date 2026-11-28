import fs = require("node:fs");
import { expect, it } from "vitest";

function loadCi() {
  return fs.readFileSync(".github/workflows/ci.yml", "utf8");
}

it("handles TypeScript import-equals and return statements", () => {
  expect(loadCi()).toContain("persist-credentials: false");
});
