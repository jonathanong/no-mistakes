import { test } from "vitest";

let shared: string[] = [];

export const spec = test.extend({});

spec("case", () => {
  shared.push("value");
});
