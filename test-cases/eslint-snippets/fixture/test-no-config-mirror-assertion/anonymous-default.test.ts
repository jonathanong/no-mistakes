import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

export default function (file: string) {
  return readFileSync(file, "utf8");
}

it("still lints after an anonymous default helper", () => {
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(ci).toContain("timeout-minutes: 15");
});
