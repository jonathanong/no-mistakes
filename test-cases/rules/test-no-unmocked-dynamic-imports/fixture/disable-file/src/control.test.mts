import { test } from "vitest";

test("loads the control target", async () => {
  await import("./sibling.mts");
});
