import { test as base } from "@acme/pw-fixtures";

export const test = process.env.CI ? base : base.extend({});
