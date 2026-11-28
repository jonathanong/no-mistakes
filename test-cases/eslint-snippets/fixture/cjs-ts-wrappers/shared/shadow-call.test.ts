import { test } from "vitest";

const shared = [];
test("case", () => {
  function update() {
    shared.push(1);
  }
  {
    const update = () => {};
    (update as () => void)();
  }
});
