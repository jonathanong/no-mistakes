import { test as base } from "@playwright/test";
import { guardTimeouts } from "./wrappers.mts";

function runnable<T>(t: T): T {
  return t;
}

export const test = runnable(guardTimeouts(base).extend({}));
