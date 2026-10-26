import { expect } from "@playwright/test";

declare const err: { message: string };

test("name", () => {
  expect(err.message).toBe("x");
  // A parameter named expect is not the imported matcher.
  function helper(expect: (value: { message: string }) => { toBe(text: string): void }) {
    expect(err.message).toBe("x");
  }
  helper(expect);
});
