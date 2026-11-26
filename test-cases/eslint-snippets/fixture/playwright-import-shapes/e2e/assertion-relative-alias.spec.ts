declare const loc: unknown;

import { expect as check } from "./fixtures";
check(loc).toBeVisible({ timeout: 30000 });

import { test as expect } from "./fixtures";
expect(loc).toBeVisible({ timeout: 30000 });
