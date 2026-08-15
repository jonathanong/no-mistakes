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

// createRequire is just as reachable through require()'s destructured or
// member-accessed result as through the static `import { createRequire }`
// form above; both forms resolve to the same require-fn-shaped capability.
function createRequireViaCommonJs() {
  const { createRequire: cr1 } = require("node:module");
  const req1 = cr1(import.meta.url);
  req1("typescript").createProgram();

  const nodeModule = require("module");
  const createRequireFn = nodeModule.createRequire;
  const req2 = createRequireFn(import.meta.url);
  req2("typescript").createProgram();
}

// A static namespace import member-accessed for `createRequire` must resolve
// the same way as the destructured/member-accessed require() forms above;
// the namespace binding is tracked as CREATE_REQUIRE_MODULES's module object
// even when nothing else about the module is configured as banned.
import * as nodeModuleNs from "node:module";
const req3 = nodeModuleNs.createRequire(import.meta.url);
req3("typescript").createProgram();

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

// Discard-on-exit alias scoping must extend past `if` to every
// conditionally or repeatedly executed construct (loop bodies, switch
// cases, try/catch, and conditional/logical branches), or an overwrite on a
// path that isn't guaranteed to run can incorrectly suppress a real report.
declare const maybeFlag: boolean;
declare const safeValue: () => void;

function loopScopeLeak() {
  let fn = createProgram;
  while (maybeFlag) {
    fn = safeValue;
  }
  return fn();
}

function switchScopeLeak() {
  let fn = createProgram;
  switch (maybeFlag) {
    case true:
      fn = safeValue;
      break;
    default:
      break;
  }
  return fn();
}

function tryScopeLeak() {
  let fn = createProgram;
  try {
    fn = safeValue;
  } catch {
    // ignore
  }
  return fn();
}

function conditionalScopeLeak() {
  let fn = createProgram;
  maybeFlag ? (fn = safeValue) : null;
  return fn();
}

function logicalScopeLeak() {
  let fn = createProgram;
  maybeFlag && (fn = safeValue);
  return fn();
}

// A switch case without a terminating break/return/throw falls through into
// the next case at runtime, so a banned reassignment made in the
// fallen-through case must still apply to a call site in the next case in
// the same run, not be reset as if that next case started fresh from the
// switch's pre-state.
function switchFallthroughLeak() {
  let fn = safeValue;
  switch (maybeFlag) {
    case true:
      fn = createProgram;
    // falls through intentionally: no break here
    case false:
      fn();
      break;
  }
}

// Member access chained directly off an inline require() call must be
// tracked without first assigning the module object to a variable.
require("typescript").createProgram();

// An inline export declaration (`export const x = ...`) exposes a tagged
// value directly; there is no `specifiers` list to inspect for this form.
import * as tsForInlineExport from "typescript";
export const inlineExportedCompile = tsForInlineExport.createProgram;

// A banned capability is just as reachable through `new` as through a plain
// call; NewExpression resolves its callee the same way CallExpression does.
import limiterForNew from "@acme/rate-limit";
new limiterForNew();
