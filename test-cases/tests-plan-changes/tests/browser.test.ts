import { expect, test } from "vitest";

test("browser fallback", () => {
  expect("example.a+b").toBe("example.a+b");
});
