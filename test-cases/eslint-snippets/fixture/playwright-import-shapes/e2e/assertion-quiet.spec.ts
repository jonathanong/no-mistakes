import { expect as check } from "chai";

declare const loc: unknown;
declare const custom: (value: unknown) => { toBeVisible(options: { timeout: number }): void };

check(loc).toBeVisible({ timeout: 30000 });
let verify = check;
verify = custom;
verify(loc).toBeVisible({ timeout: 30000 });
