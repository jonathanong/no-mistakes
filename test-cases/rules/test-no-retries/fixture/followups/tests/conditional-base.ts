import { test as base } from "@playwright/test";
import { base as localBase } from "./local-base";

export const test = process.env.CI ? base : base.extend({});
export const localTest = process.env.CI ? localBase : localBase.extend({});
export const chainedTest = process.env.CI ? localTest : localTest.extend({});
