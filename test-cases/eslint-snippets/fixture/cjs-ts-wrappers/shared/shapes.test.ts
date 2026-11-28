const bag = [];
const { test: t, expect: e, beforeEach, ...rest } = require("vitest");
t("records", () => {
  bag.push(1);
});
(0, t)("comma", () => {
  bag.push(1);
});
void e;
void rest;

const { "test": literal } = require("vitest");
const literalBag = [];
literal("literal", () => {
  literalBag.push(1);
});

const { ["test"]: computed } = require("vitest");
const computedBag = [];
computed("computed", () => {
  computedBag.push(1);
});

const { test: nested } = require("vitest");
void nested;
const { test: { skipped } } = require("vitest");
void skipped;

const pw = require("@playwright/test");
const pages = [];
pw("opens", () => {
  pages.push(1);
});

const commaBag = [];
const commaTest = (0, t);
commaTest("comma alias", () => {
  commaBag.push(1);
});

const callBag = [];
const callTest = t.call;
callTest(null, "call alias", () => {
  callBag.push(1);
});

const resetBag = [];
(0, beforeEach)(() => {
  resetBag.length = 0;
});
t("reset", () => {
  resetBag.push(1);
});

const arrayBag = [];
const [arrayTest] = require("vitest");
arrayTest("array", () => {
  arrayBag.push(1);
});
