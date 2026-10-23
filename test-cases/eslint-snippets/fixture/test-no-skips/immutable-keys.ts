import { test as vitest, describe } from "vitest";
import pw from "@playwright/test";
const modifier = "skip";
const alias = modifier;
const suiteKey = `describe`;
const extendKey = "extend";
vitest[modifier]("Vitest const key", () => {});
pw[alias]("Playwright const alias", () => {});
const check = vitest;
check[alias]("runner alias", () => {});
const extended = pw[extendKey]({});
extended[modifier]("computed extend", () => {});
vitest[extendKey]({})[alias]("Vitest extended", () => {});
pw[suiteKey][modifier]("computed suite chain", () => {});
describe[alias].each([1])("table", () => {});
{
  const modifier = "only";
  pw[modifier]("inner binding", () => {});
}
vitest[modifier]("outer binding remains", () => {});
// no-mistakes-disable-next-line test-no-skips -- reviewed fixture
pw[modifier]("suppressed computed key", () => {});
