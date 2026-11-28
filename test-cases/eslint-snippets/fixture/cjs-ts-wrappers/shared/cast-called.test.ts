import { test } from "vitest";

const shared: number[] = [];
test("case", () => {
  const update = (() => shared.push(1)) as () => void;
  update();
  ((() => shared.push(2)) as () => void)();
});
