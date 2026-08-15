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

// These destructures are intentionally unsupported as banned-import aliases,
// matching the reference rule's precision boundary for nested/rest patterns.
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
