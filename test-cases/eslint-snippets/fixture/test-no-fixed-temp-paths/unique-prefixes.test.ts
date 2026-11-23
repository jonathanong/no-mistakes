import fs, { mkdtempDisposableSync as makeDisposable, mkdtempSync } from "node:fs";
import * as promises from "node:fs/promises";
import { mkdtempDisposable as makeAsyncDisposable, mkdtemp } from "fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

const literal = "/tmp/prefix-";
const joined = join(tmpdir(), "prefix-");
const resolved = resolve(tmpdir(), "prefix-");
mkdtempSync(literal);
await mkdtemp(joined);
fs.mkdtemp(resolved, callback);
await fs.promises.mkdtemp(resolved);

using direct = fs.mkdtempDisposableSync("/tmp/direct-");
using named = makeDisposable(literal);
using required = require("fs").mkdtempDisposableSync(joined);
await using asyncDirect = await promises.mkdtempDisposable("/var/tmp/async-direct-");
await using asyncNamed = await makeAsyncDisposable(resolved);
await using fromFs = await fs.promises.mkdtempDisposable(literal);
await using fromRequired = await require("node:fs/promises").mkdtempDisposable(joined);

const usedAsPath = join(tmpdir(), "also-read-");
await mkdtemp(usedAsPath);
fs.readFileSync(usedAsPath);

import * as fsNamespace from "node:fs";
using namespaced = fsNamespace.mkdtempDisposableSync(resolved);
