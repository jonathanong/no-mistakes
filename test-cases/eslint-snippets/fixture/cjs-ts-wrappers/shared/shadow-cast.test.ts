import * as v from "vitest";

const items: number[] = [];
v.test("outer", () => {
  items.push(1);
});

function helper() {
  const v = {
    test(_name: string, fn: () => void) {
      fn();
    },
  };
  (v as any).test("inner", () => {
    items.push(2);
  });
}
void helper;
