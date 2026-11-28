import { expect, it } from "vitest";

declare const spec: string;
const dynamicFs = require(spec);

it("does not crash on a non-literal require specifier", () => {
  expect(dynamicFs.realpathSync("package.json")).toContain("package");
});
