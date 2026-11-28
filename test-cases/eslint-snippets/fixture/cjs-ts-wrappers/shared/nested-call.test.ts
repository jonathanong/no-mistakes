import { test } from "vitest";

const bag = [];
test("case", () => {
  function update() {
    bag.push(1);
  }
  (consume as any)(update());
});
