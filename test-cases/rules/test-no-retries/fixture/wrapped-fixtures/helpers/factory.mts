import { test as base } from "@playwright/test";

export function createTest(_options?: { user?: string }) {
  return base.extend({});
}

export function createOpaqueTest(_options?: { user?: string }) {
  return Object.assign({}, base);
}
