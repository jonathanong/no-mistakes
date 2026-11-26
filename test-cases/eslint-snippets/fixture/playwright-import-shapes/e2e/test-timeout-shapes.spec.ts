import { expect, test } from "@playwright/test";
import * as pw from "@playwright/test";

declare const other: unknown;

const extended = test.extend({});
extended.setTimeout(999_999);

pw.locator(".x");
expect(1);

const { expect: check, test: aliased } = require("@playwright/test");
aliased.setTimeout(999_999);
void check;

test("pattern", (_unused: unknown, { setTimeout, slow }: any) => {
  setTimeout(999_999);
  void slow;
});

test("rest", (_unused: unknown, ...rest: unknown[]) => {
  void rest;
});

test("bound", () => {
  test.setTimeout.bind(other)(1);
});
