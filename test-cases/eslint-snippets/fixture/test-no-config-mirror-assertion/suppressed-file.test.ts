// no-mistakes-disable-file test-no-config-mirror-assertion -- intentional workflow security assertions
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("is suppressed across the file with a reason", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
});
