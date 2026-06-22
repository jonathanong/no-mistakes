import { test } from "vitest";

const myTest = test.extend({});
let items: string[] = [];

myTest.beforeEach(() => {
  items = [];
});

myTest("uses setup cleanup from an extended test alias", () => {
  items.push("ok");
});
