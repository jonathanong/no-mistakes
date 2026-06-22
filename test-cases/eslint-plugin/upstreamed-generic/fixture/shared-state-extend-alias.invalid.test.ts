import { test, test as base } from "vitest";

let direct = 0;
let renamed = 0;
let chained = 0;
let described = 0;
let tabled = 0;

const myTest = test.extend({});
const renamedTest = base.extend({});
const chainedTest = base.extend({}).extend({});

myTest("case", () => {
  direct++;
});

renamedTest.only("case", () => {
  renamed++;
});

chainedTest("case", () => {
  chained++;
});

renamedTest.describe("suite", () => {
  described++;
});

myTest.each([1])("case %s", () => {
  tabled++;
});
