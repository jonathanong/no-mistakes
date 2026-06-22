import { describe as suite, test, beforeEach } from "vitest";

let items: string[] = [];

suite.only("scoped cleanup", () => {
  beforeEach(() => {
    items = [];
  });

  test("inside", () => {
    items.push("inside");
  });
});

test("outside", () => {
  items.push("outside");
});
