declare const loc: unknown;

import { expect } from "@playwright/test";
const slow = expect.configure({ timeout: 1000 });
slow(loc).toBeVisible({ timeout: 30000 });

const soft = expect.soft;
soft(loc).toBeVisible({ timeout: 30000 });

import { makeExpect } from "./assertions";
const check = makeExpect();
check(loc).toBeVisible({ timeout: 30000 });
