import { test } from "vitest";
const items: string[] = [];
function run() {
  items.push("wrapped");
}
function plain() {
  items.push("plain");
}
const timeout = 1;
test("wrapped", run, timeout as number);
test("plain", plain, timeout);
