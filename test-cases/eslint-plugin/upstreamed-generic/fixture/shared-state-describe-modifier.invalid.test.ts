import { describe, test, beforeEach } from "vitest";

let items: string[] = [];

describe.only("scoped cleanup", () => {
  beforeEach(() => {
    items = [];
  });

  test("inside the cleaned suite", () => {
    items.push("inside");
  });
});

test("outside the cleaned suite", () => {
  items.push("outside");
});
