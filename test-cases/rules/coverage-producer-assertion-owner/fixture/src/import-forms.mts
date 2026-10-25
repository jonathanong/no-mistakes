import assert from 'node:assert';
import * as assertions from 'node:assert';
import { strictEqual as equal } from 'node:assert';
import { existsSync } from 'node:fs';

export function namedAssertion() { equal(1, 1); }
export function namespaceAssertion() { assertions.strictEqual(1, 1); }
// Default-member resolution is deliberately uncertain, never assertion proof.
export function defaultMemberAssertion() { assert.strictEqual(1, 1); }
// A reachable assertion does not erase an unmodeled external call.
export function externalAndAssertion() { existsSync('.'); equal(1, 1); }
