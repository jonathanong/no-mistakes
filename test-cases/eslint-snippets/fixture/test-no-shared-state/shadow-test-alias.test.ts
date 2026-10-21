import * as v from "vitest";

const items = new Map<string, number>();
const check = v.test;

function helper(check: (name: string, fn: () => void) => void) {
  check("ordinary", () => {
    items.set("shadow", 1);
  });
}

check("real", () => {
  items.set("a", 1);
});
helper(check);
