declare const loc: unknown;

const pw = require("@playwright/test");
pw.expect(loc).toBeVisible({ timeout: 30000 });

const { expect: e } = require("@playwright/test");
e(loc).toBeVisible({ timeout: 30000 });

import eq = require("@playwright/test");
eq.expect(loc).toBeVisible({ timeout: 30000 });

import { "expect" as named } from "@playwright/test";
named(loc).toBeVisible({ timeout: 30000 });
