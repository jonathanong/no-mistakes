import { test as base } from "@playwright/test";

export function createTest(_options: { baseURL: string }) {
  return base.extend({});
}
