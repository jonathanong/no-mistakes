declare const loc: unknown;

import { expect } from "./fixtures";
expect(loc).toBeVisible({ timeout: 30000 });
expect(loc).toBeVisible({ timeout: 1000 });
expect(loc).toBeVisible();
