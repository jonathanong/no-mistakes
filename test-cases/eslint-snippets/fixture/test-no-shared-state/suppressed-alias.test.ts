import * as v from "vitest";

const items = new Map<string, number>();
const check = v.test;

check("real", () => {
  // no-mistakes-disable-next-line test-no-shared-state -- retained probe
  items.set("a", 1);
  items.set("b", 1);
});
