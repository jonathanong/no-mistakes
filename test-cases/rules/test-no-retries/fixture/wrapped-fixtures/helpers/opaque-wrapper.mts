import { test as base } from "@playwright/test";

function wrapDynamic(t: typeof base) {
  return Object.assign({}, t);
}

export const test = wrapDynamic(base);
