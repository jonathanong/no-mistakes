import { it } from "vitest";

it("clears the root before importing the scanner", async () => {
  delete process.env.EXAMPLE_LOCAL_DIR;
  await import("./scan.mjs");
});
