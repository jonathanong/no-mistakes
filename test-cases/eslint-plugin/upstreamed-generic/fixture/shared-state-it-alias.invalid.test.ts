import { it as spec } from "vitest";

const shared: string[] = [];

spec("case", () => {
  shared.push("value");
});
