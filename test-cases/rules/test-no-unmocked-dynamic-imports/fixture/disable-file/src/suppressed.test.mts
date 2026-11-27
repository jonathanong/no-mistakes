// no-mistakes-disable-file test-no-unmocked-dynamic-imports
import { test } from "vitest";

test("loads the suppressed target", async () => {
  await import("./selected.mts");
});
