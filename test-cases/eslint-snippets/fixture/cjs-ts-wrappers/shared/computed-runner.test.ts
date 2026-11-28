import vt from "vitest";
import { test } from "vitest";

const items: number[] = [];
vt["test"]("computed", () => {
  items.push(1);
});
vt.test("member", () => {
  items.push(2);
});
test["only"]("kept", () => {
  items.push(3);
});
