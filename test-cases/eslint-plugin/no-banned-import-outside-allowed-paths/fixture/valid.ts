import { createProgram as otherCreateProgram } from "some-other-compiler";
otherCreateProgram();

import { transpileModule } from "typescript";
transpileModule("");

// The top-level `createProgram` import is banned, but every call site below
// resolves to the shadowing function parameter of the same name, not the
// import binding.
import { createProgram } from "typescript";

function withParameter(createProgram: () => void) {
  return createProgram();
}

declare function createFactory(): () => void;
const createProgram2 = createFactory();
createProgram2();

import * as ts from "typescript";
ts.transpileModule("");

import * as ts2 from "typescript";
declare const key: string;
const dynamic = ts2[key];
dynamic();

// A nested destructure target is intentionally unsupported as a banned-import
// alias, matching the reference rule's precision boundary. The rest binding
// below does carry the module tag, but it is never used, so nothing is reported.
import * as ts3 from "typescript";
const {
  createProgram: {},
} = ts3;
const { unrelated, ...tsRestUnused } = ts3;

// A parameter named `require` shadows Node's global require, so this is not
// treated as a require() call regardless of the literal argument.
function withLocalRequire(require: (specifier: string) => unknown) {
  const mod = require("typescript");
  return mod;
}

// A non-literal or unbanned require()/import() specifier can't be resolved
// to a module and is not tracked.
declare const dynamicSpecifier: string;
const dynRequire = require(dynamicSpecifier);
const unbannedRequire = require("unbanned-module");
const dynImport = import(dynamicSpecifier);
const unbannedImport = import("unbanned-module");

// Only one level of member access off a tracked binding is resolved; a
// namespace import wrapped in another object is not tracked further.
import * as tsChain from "typescript";
const wrapped = { inner: tsChain };
wrapped.inner.createProgram();

// Spreading a value that isn't itself a tracked module object contributes no
// banned modules to the merged tag.
declare const unrelatedValue: object;
const spreadOfUnrelated = { ...unrelatedValue };

// Type-only imports and exports are erased at compile time, so they carry no
// runtime reference and are never tracked or reported, even for otherwise
// banned names.
import type { createProgram as typeOnlyImport } from "typescript";
import { type createWatchProgram as typeOnlySpecifierImport } from "typescript";
export type { createProgram } from "typescript";
export { type createWatchProgram } from "typescript";
export type { typeOnlyImport };

// Assigning to a name with no resolvable binding does not crash the alias
// tracker; it is silently ignored.
undeclaredGlobal = require("typescript");
undeclaredGlobal.createProgram();

// Destructuring (including via rest) from a non-tracked source contributes
// no tags, even when a destructured name matches a banned name elsewhere.
const { createProgram: notBanned, ...restUnbanned } = require("unbanned-module");

// A computed, non-literal pattern key can't be resolved to a static property
// name and is not tracked.
import * as ts4 from "typescript";
const { [`computed`]: computedVal } = ts4;

// A named import that isn't itself banned carries no tag, so re-exporting it
// is not reported.
import { transpileModule as reExportedTranspile } from "typescript";
export { reExportedTranspile };

// A namespace import of a module with no banned names configured carries no
// tag.
import * as unrelatedNs from "unrelated-namespace-module";
unrelatedNs.whatever();

// Calling a namespace-imported module object directly is only reported when
// the module's default export is specifically banned; "typescript" bans
// named exports here, not "default".
import * as ts5 from "typescript";
ts5();

// An alias unconditionally overwritten to something untracked before being
// re-exported must not "revive" its earlier tag from the forward
// fixed-point pass; only a genuine forward reference (never touched in
// real time) may fall back to a forward tag.
import * as ts6 from "typescript";
let reassignedBeforeExport = ts6.createProgram;
reassignedBeforeExport = ts6.transpileModule;
export { reassignedBeforeExport };

// The same stale-forward-tag fallback must not apply to default exports
// either.
import * as ts7 from "typescript";
let reassignedBeforeDefaultExport = ts7.createProgram;
reassignedBeforeDefaultExport = ts7.transpileModule;
export default reassignedBeforeDefaultExport;

// An existing tagged alias overwritten through a nested object-destructure
// or an array-destructure must lose its stale tag, even though neither
// destructure shape resolves to a new tag itself.
import * as ts8 from "typescript";
declare const safeNested: { value: { createProgram: () => void } };
declare const safeArray: [() => void];
let nestedOverwrite = ts8.createProgram;
({
  value: { createProgram: nestedOverwrite },
} = safeNested);
nestedOverwrite();
let arrayOverwrite = ts8.createProgram;
[arrayOverwrite] = safeArray;
arrayOverwrite();

// An unaliased export-star from a module with no banned names configured at
// all carries no tag, the same as one banned only on "default".
export * from "unrelated-namespace-module";

// An empty switch (no cases at all) still pushes and pops scope-tracking
// state around it; nothing inside ever assigns a banned alias, so nothing is
// reported.
declare const emptySwitchFlag: boolean;
switch (emptySwitchFlag) {
}

// A `break` nested directly inside a bare block at the end of a case body is
// still a guaranteed terminator for that case (the block's statements run
// unconditionally, same as a direct case-body child), so it must not be
// treated as falling through into the next case.
declare const switchBlockFlag: boolean;
declare const safeFn: () => void;
function switchBlockTerminatorNoLeak() {
  let fn = safeFn;
  switch (switchBlockFlag) {
    case true: {
      fn = createProgram;
      break;
    }
    case false:
      fn();
      break;
  }
}

// A locally re-exported name with no resolvable binding does not crash the
// export-tag resolver; it is silently ignored, matching the alias tracker's
// handling of undeclared globals elsewhere in this file.
export { undeclaredExportedGlobal };

// Only the specific `createRequire` property off a require()'d
// "node:module"/"module" binding is treated as Node's createRequire; any
// other property reached the same way is not, since it isn't itself a
// configured banned name.
const nodeModuleBuiltins = require("node:module");
nodeModuleBuiltins.builtinModules;

// The same precision boundary applies to a static namespace import of
// "node:module"/"module": only its `createRequire` property is treated as
// Node's createRequire, not any other property reached the same way.
import * as nodeModuleNsBuiltins from "node:module";
nodeModuleNsBuiltins.builtinModules;

// A destructured inline export declaration is unsupported for tagging (only
// a plain identifier declarator is), matching the destructuring precision
// boundary elsewhere in this file.
import * as ts9 from "typescript";
export const { createProgram: destructuredInlineExport } = ts9;

// A directly re-exported name from a `from` source that isn't itself banned
// carries no report, matching the un-sourced local re-export precision
// boundary above.
export { transpileModule as reExportedTranspileFromSource } from "typescript";
