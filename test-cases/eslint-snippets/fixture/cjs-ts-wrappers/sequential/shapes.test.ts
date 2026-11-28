import { "test" as quotedTest } from "vitest";
quotedTest.sequential("quoted", () => {});

import fsEq = require("node:fs");
void fsEq;
import { test as nodeTest } from "node:test";
nodeTest.sequential("node test", () => {});

const spec = "vitest";
import vtEq = require("vitest");
vtEq.sequential("equals sequential is not a runner", () => {});
const { test: fromEquals } = vtEq;
fromEquals.sequential("from equals", () => {});

import vtDefault from "vitest";
const { test: fromDefault } = vtDefault;
fromDefault.sequential("from default", () => {});

const { test: restTest, ...ignored } = require("vitest");
restTest.sequential("rest", () => {});
const { "test": literalKey } = require("vitest");
literalKey.sequential("literal key", () => {});
const { ["test"]: computedKey } = require("vitest");
computedKey.sequential("computed key", () => {});
const { [spec]: dynamicLocal } = require("vitest");
dynamicLocal.sequential("computed identifier", () => {});

const fs = require("node:fs");
void fs;
const { test: dynPattern } = require(spec);
dynPattern.sequential("dynamic pattern", () => {});
const { [spec]: dynComputed } = require(spec);
dynComputed.sequential("dynamic computed pattern", () => {});

const vt = require("vitest");
vt.sequential("require sequential is not a runner", () => {});
const { test: fromRequire } = vt;
fromRequire.sequential("from require", () => {});

const { expect: e, ...expectRest } = require("vitest");
e.sequential("expect", () => {});
import { "expect" as quotedExpect } from "vitest";
quotedExpect.sequential("quoted expect", () => {});
const [arrayVt] = require("vitest");
arrayVt.sequential("array", () => {});
const { test: [first] } = require("vitest");
void first;
void ignored;
void expectRest;
