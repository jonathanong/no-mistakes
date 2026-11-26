declare const loc: unknown;

import * as chai from "chai";
chai.expect(loc).toBeVisible({ timeout: 30000 });

import view from "react";
view.expect(loc).toBeVisible({ timeout: 30000 });

import * as pw from "@playwright/test";
pw["expect"](loc).toBeVisible({ timeout: 30000 });

import { "" as blank } from "@playwright/test";
blank(loc).toBeVisible({ timeout: 30000 });
