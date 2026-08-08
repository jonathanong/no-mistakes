import { test } from "vitest";

test("filesystem skips remain outside dynamic-import reachability", async () => {
  await import("../skipped/target");
});
