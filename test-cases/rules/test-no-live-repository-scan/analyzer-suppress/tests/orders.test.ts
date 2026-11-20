import { execSync } from "node:child_process";
import { test } from "vitest";

test("suppressed analyzer launch", () => {
  // no-mistakes-disable-next-line test-no-live-repository-scan -- fixture root owned outside this example
  execSync("widget-audit check");
  execSync("pnpm exec widget-audit check");
});
