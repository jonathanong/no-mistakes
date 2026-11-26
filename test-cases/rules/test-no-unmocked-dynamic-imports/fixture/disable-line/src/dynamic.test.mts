import { test } from "vitest";

test("loads the selected target", async () => {
  await import("./selected.mts"); // no-mistakes-disable-line test-no-unmocked-dynamic-imports
});

test("loads the sibling target", async () => {
  await import("./sibling.mts");
});
