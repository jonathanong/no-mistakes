import { test as base } from "@playwright/test";

export const { test } = { test: base.extend({}) };
