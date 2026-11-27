declare const loc: unknown;
declare const name: string;

import * as expect from "@playwright/test";
expect(loc).toBeVisible({ timeout: 30000 });

import pw from "@playwright/test";
pw.expect(loc).toBeVisible({ timeout: 30000 });

import * as local from "./helpers";
local.expect(loc).toBeVisible({ timeout: 30000 });

const { ["expect"]: named } = require("@playwright/test");
named(loc).toBeVisible({ timeout: 30000 });

const [check] = require("@playwright/test");
check(loc).toBeVisible({ timeout: 30000 });

import * as api from "@playwright/test";
api[name](loc).toBeVisible({ timeout: 30000 });

function shadowed() {
  function require(id: string) {
    return {};
  }
  declare const mod: string;
  const dynamic = require(mod);
  dynamic.expect(loc).toBeVisible({ timeout: 30000 });
}
void shadowed;
