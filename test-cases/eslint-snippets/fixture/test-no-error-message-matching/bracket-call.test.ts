import { expect as check } from "vitest";

declare const err: { message: string };
declare const name: string;

test("bracket", () => {
  check["call"](null, err.message).toMatch("x");
  check["apply"](null, [err.message]).toMatch("x");
  check[name](null, err.message).toMatch("x");
});
