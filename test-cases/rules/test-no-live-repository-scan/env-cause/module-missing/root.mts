import { it } from "vitest";

it("imports the scanner without assigning its environment root", async () => {
  await import("./scan.mjs");
});
