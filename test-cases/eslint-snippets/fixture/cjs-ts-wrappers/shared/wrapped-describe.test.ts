import { beforeEach, describe, test } from "vitest";
import { test as pwTest } from "@playwright/test";

let suiteItems: string[] = [];
(describe as typeof describe)("wrapped suite", () => {
  beforeEach(() => {
    suiteItems = [];
  });
  test("pushes inside wrapped suite", () => {
    suiteItems.push("a");
  });
});

test("pushes outside wrapped suite without cleanup", () => {
  suiteItems.push("b");
});

let serialItems: string[] = [];
(pwTest.describe as typeof pwTest.describe)("wrapped playwright suite", () => {
  (pwTest.describe.configure as typeof pwTest.describe.configure)({ mode: "serial" });
  pwTest.beforeAll(() => {
    serialItems = [];
  });
  pwTest("pushes in configured serial suite", () => {
    serialItems.push("a");
  });
});

let serialShorthandItems: string[] = [];
(pwTest.describe.serial as typeof pwTest.describe.serial)("wrapped serial suite", () => {
  pwTest.beforeAll(() => {
    serialShorthandItems = [];
  });
  pwTest("pushes in shorthand serial suite", () => {
    serialShorthandItems.push("a");
  });
});
