import { test as base } from "@playwright/test";

declare const useBase: boolean;
const replacement = { describe: { configure(_options: object) {} } };

export function makeTest(_fixture?: typeof base) {
  if (useBase) return base;
  return replacement;
}

export function makeProvenTest() {
  return base.extend({});
}
