import { expect } from "vitest";

declare const err: { message: string };
declare const name: string;

test("matcher", () => {
  expect(err.message).toMatch["call"](null, "x");
  expect(err.message).toMatch[`apply`](null, ["x"]);
  expect(err.message).toMatch[name](null, "x");
});
