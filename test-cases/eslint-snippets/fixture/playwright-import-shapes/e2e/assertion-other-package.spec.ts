declare const loc: unknown;

import { expect } from "@playwright/experimental-ct-react";
expect(loc).toBeVisible({ timeout: 30000 });

import { expect as check } from "@playwright/experimental-ct-react";
check(loc).toBeVisible({ timeout: 30000 });
