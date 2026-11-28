import * as pw from "@playwright/test";
const { test: check } = pw;
check.skip("direct", () => {});
(check as typeof check).describe(
  "suite",
  { annotation: { type: "skip", description: "disabled" } },
  () => {},
);
