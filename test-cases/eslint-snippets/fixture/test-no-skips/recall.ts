import pw from "@playwright/test";
import vitest from "vitest";
import * as v from "vitest";

pw.skip("default export is Playwright test", () => {});
pw[`skip`]("expression-free template", () => {});
const { test } = v;
test.skip("destructured namespace member", () => {});
test["skip"]("static bracket", () => {});
// Vitest has no default export, so this name is not the runner.
vitest.skip("not a runner", () => {});
// no-mistakes-disable-next-line test-no-skips -- retained probe
test.skip("suppressed", () => {});
