lateImport();
import { lateImport } from "./late";
import { createProgram as makeProgram } from "typescript";
import { tooling as namedTooling } from "tooling";
import * as path from "node:path";
import { createRequire } from "node:module";

function local() {}
class LocalClass {
  constructor() {
    makeProgram([]);
  }
}
const makeProgramAlias = makeProgram;
const require = createRequire(import.meta.url);
const requiredTypeScript = require("typescript");
const commonJsTypeScript = require("typescript");
const { createProgram: destructuredProgram } = require("typescript");
const directMemberProgram = require("typescript").createProgram;

// The call appears before the declaration: program-scope hoisting must keep
// this a local callable rather than a fabricated global identity.
hoistedTopLevel();
function hoistedTopLevel() {}

const topLevelValue = null;
topLevelValue();

{
  const blockValue = null;
  blockValue();
}

{
  function blockCallable() {}
  blockCallable();
}

export const exportedApi = {
  run() {
    makeProgram([]);
  },
};

export function run() {
  makeProgram([]);
  makeProgramAlias([]);
  requiredTypeScript.createProgram([]);
  commonJsTypeScript.createProgram([]);
  commonJsTypeScript["createProgram"]([]);
  destructuredProgram([]);
  directMemberProgram([]);
  namedTooling.createProgram([]);
  require("typescript").createProgram([]);
  path.resolve(".");
  path.posix.resolve(".");
  local();
  globalThis.setTimeout(() => {}, 1);
  new globalThis.URL("https://example.com");
  (() => {})();
  (() => makeProgram([]))();
}

export function constructLocal() {
  new LocalClass();
}

export function shadowed(makeProgram: () => void) {
  makeProgram();
}

export function shadowedRequire(require: (name: string) => unknown) {
  const localModule = require("typescript");
  localModule.createProgram([]);
}

export function localAliases() {
  const first = makeProgram;
  const second = first;
  second([]);
}

export function shadowedAlias() {
  const first = makeProgram;
  {
    // The inner binding intentionally hides the imported alias chain.
    const first = () => {};
    first();
  }
}

export function lexicalTdz() {
  {
    makeProgram();
    const makeProgram = () => {};
    makeProgram();
  }
}

export function functionBodyTdz() {
  makeProgram();
  const makeProgram = () => {};
  makeProgram();
}

export const arrowBodyTdz = () => {
  makeProgram();
  const makeProgram = () => {};
};

export const functionExpressionBodyTdz = function () {
  makeProgram();
  var makeProgram = null;
};

export function localClassConstructor() {
  class Worker {
    constructor() {
      makeProgram([]);
    }
  }
  new Worker();
}

export function loopBindingsDoNotLeak() {
  for (let makeProgram = () => {}; false; ) {
    makeProgram();
  }
  makeProgram([]);
  for (const makeProgram of []) {
    makeProgram();
  }
  makeProgram([]);
  for (const makeProgram in {}) {
    makeProgram();
  }
  makeProgram([]);
  switch (0) {
    case 0:
      const makeProgram = () => {};
      makeProgram();
      break;
  }
  makeProgram([]);
}

function outerCallable() {}

export function parameterStopsOuterCallable(outerCallable: () => void) {
  outerCallable();
}

export function localStopsOuterCallable() {
  const outerCallable = null;
  outerCallable();
}
