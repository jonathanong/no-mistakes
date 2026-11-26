import { test as base } from "@playwright/test";

export function createTest() {
  return base.extend({});
}
