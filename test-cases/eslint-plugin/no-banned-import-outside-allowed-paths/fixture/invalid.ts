import { createProgram } from "typescript";
createProgram();

import { createWatchProgram } from "typescript";
const watch = createWatchProgram;
watch();

import * as ts from "typescript";
ts.createProgram();

import * as ts2 from "typescript";
const { createProgram: cp } = ts2;
cp();

import limiter from "@acme/rate-limit";
limiter();

import * as rateLimit from "@acme/rate-limit";
rateLimit.invalidateAll();

const { createProgram: cp2 } = require("typescript");
cp2();

const ts3 = require("typescript");
ts3.createProgram();

import { createRequire } from "node:module";
const req = createRequire(import.meta.url);
const tsReq = req("typescript");
tsReq.createProgram();

async function loadTs() {
  const { createProgram: dynCp } = await import("typescript");
  dynCp();
}

import * as tsAlias from "typescript";
const tsAlias2 = tsAlias;
tsAlias2.createProgram();

export { createProgram } from "typescript";

import { createWatchProgram as cwp } from "typescript";
export { cwp };

import limiterForExport from "@acme/rate-limit";
export default limiterForExport;

export * from "typescript";

import * as tsNs from "typescript";
const merged = { extra: 1, ...tsNs };
merged.createProgram();

const { unrelatedExport, ...tsRest } = require("typescript");
tsRest.createProgram();

declare const flag: boolean;
function conditionalRequire() {
  let mod;
  if (flag) {
    mod = require("typescript");
    mod.createProgram();
  }
}

// Forward reference: `useLater` is declared before `laterTs`, but the
// require()-derived alias is still reachable because the rule pre-seeds
// module-scope aliases in a fixed-point pass before checking call sites.
function useLater() {
  return laterTs.createProgram();
}

const laterTs = require("typescript");

// Top-level non-`=` assignments must not crash the alias tracker (real-time
// or the forward fixed-point pass) and must not be treated as aliasing.
let counter = 0;
counter += 1;
const counterHolder = { count: 0 };
counterHolder.count += 1;

// Assignment-expression destructuring (not a `const`/`let` declarator) is a
// separate binding path from `VariableDeclarator` and must be tracked by
// both the real-time recorder and the forward fixed-point pass.
let cp3;
({ createProgram: cp3 } = require("typescript"));
cp3();

// IIFEs do not push a new function alias scope (matching the reference
// rule's traversal), so a banned alias assigned inside one is tracked
// exactly like top-level code, not scoped away.
(function () {
  const tsIife = require("typescript");
  tsIife.createProgram();
})();
