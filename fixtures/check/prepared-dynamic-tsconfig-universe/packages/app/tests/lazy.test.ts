import { test } from "vitest";

test("loads the generated package alias", async () => {
  await import("@app/lazy");
});
