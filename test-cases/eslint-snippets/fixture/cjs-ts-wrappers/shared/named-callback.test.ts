import { test } from "vitest";
const items: string[] = [];
function run() {
  items.push("x");
}
test("asserted", run as typeof run);
test("non-null", run!);
