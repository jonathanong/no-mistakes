import assert from 'node:assert';
import * as assertions from 'node:assert';
import { strictEqual as equal } from 'node:assert';
import { hostname } from 'node:os';
import { query } from 'pg';
import strictAssert from 'node:assert/strict';
import * as strictNamespace from 'node:assert/strict';
import looseAssert from 'assert';
import strictPackage from 'assert/strict';
import { execFile } from 'node:child_process';

export function namedAssertion() { equal(1, 1); }
export function namespaceAssertion() { assertions.strictEqual(1, 1); }
export function defaultMemberAssertion() { assert.strictEqual(1, 1); }
export function externalAndAssertion() { hostname(); query('select 1'); equal(1, 1); }
export function externalOnly() { hostname(); query('select 1'); }
export function strictDefaultEqual() { strictAssert.equal(1, 1); }
export function strictNamespaceOk() { strictNamespace.ok(1); }
export function bareStrictAssert() { strictAssert(1); }
export function bareLooseAssert() { looseAssert(1); }
export function strictPackageEqual() { strictPackage.equal(1, 1); }
export function bareNodeAssert() { assert(1); }
export function otherChildExport() { execFile('true'); }
