import { test, beforeEach } from "vitest";
const items = [] as string[];
test("collects", () => {
  items.push("x");
});
let list: string[] = [];
beforeEach(() => {
  list = [] as string[];
});
test("resets", () => {
  list.push("y");
});
const bag: any[] = [];
test("asserted element", () => {
  (bag as any[]).push(1);
});
test("non-null element", () => {
  bag!.push(1);
});
test("asserted method", () => {
  (bag.push as any)(1);
});
