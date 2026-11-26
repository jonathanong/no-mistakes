import type { test } from "@playwright/test";

declare function makeFixture(): typeof test;
export const customTest = makeFixture();
