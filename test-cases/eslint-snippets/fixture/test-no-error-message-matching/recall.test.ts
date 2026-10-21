import { expect as check } from "vitest";
import { expect as pwExpect } from "@playwright/test";
import * as v from "vitest";

declare const err: { message: string; body: { message: string } };
declare const label: string;

const seen = check;
const { expect } = v;
const { expect: cjsExpect } = require("vitest");

test("messages", () => {
  check(err["message"]).toMatch("x");
  check(err[`message`]).toEqual("x");
  check?.(err.message).toMatch("x");
  seen(err.message).toMatch("x");
  v.expect(err.message).toMatch("x");
  v["expect"](err.message).toMatch("x");
  expect(err.message)["toMatch"]("x");
  expect(err.message)[`toMatch`]("x");
  (0, expect(err.message).toMatch)("x");
  expect(err.message).toMatch.call(null, "x");
  check.call(null, err.message).toMatch("x");
  const ns = v;
  ns.expect(err.message).toMatch("x");
  pwExpect(err.message).toBe("x");
  cjsExpect(err.message).toMatch("x");
  function helper(expect: typeof check) {
    expect(err.message).toBe("x");
  }
  expect(err[label]).toMatch("x");
  expect(err.body.message).toMatch("x");
  // no-mistakes-disable-next-line test-no-error-message-matching -- retained probe
  expect(err.message).toMatch("kept");
});
