import { test } from "vitest";
import { test as browserTest } from "@playwright/test";

test("destructured", ({ skip }) => skip());
test("renamed", ({ skip: omit }) => omit());
test("defaulted", ({ skip = () => {} }) => skip());
test("renamed and defaulted", ({ skip: omit = () => {} }) => omit());
test("captured", (ctx) => {
  const omit = ctx.skip;
  omit();
});
test("captured alias", (ctx) => {
  const omit = ctx.skip;
  const later = omit;
  later();
});
test("local destructure", (ctx) => {
  const { skip: omit = () => {} } = ctx;
  omit();
});
test.for([1])("second context", (_, { skip: omit }) => omit());
test.for([1])("second captured", (_, ctx) => {
  const omit = ctx.skip;
  omit();
});

test.each([{ skip() {} }])("row data", ({ skip }) => skip());
test.for([{ skip() {} }])("first row data", ({ skip }, ctx) => {
  skip();
});
test("shadowed", ({ skip }) => {
  function nested(skip: () => void) {
    skip();
  }
  nested(() => {});
});
test("mutable", (ctx) => {
  let omit = ctx.skip;
  omit();
  omit = () => {};
});
test("other object", (ctx) => {
  const other = { skip() {} };
  const omit = other.skip;
  omit();
});
test("other destructure", (ctx) => {
  const other = { skip() {} };
  const { skip: omit } = other;
  omit();
});
browserTest("playwright fixture", async ({ skip }) => {
  skip();
});
