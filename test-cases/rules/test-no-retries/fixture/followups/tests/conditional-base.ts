import { test as base } from "@playwright/test";

export const test = process.env.CI ? base : base.extend({});
