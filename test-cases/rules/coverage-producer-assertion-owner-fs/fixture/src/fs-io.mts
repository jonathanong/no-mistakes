import fsDefault from 'node:fs';
import bareFs from 'fs';
import * as fsNs from 'node:fs';
import { readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { readFile as bareRead } from 'fs/promises';
import { equal } from 'node:assert/strict';
import { hostname } from 'node:os';

// Control 3: fs producer, asserted -> passes.
export function fsAsserted() { equal(readFileSync('result', 'utf8'), 'expected'); }
// Control 4: fs producer, unasserted -> no-owner finding.
export function fsUnasserted() { readFileSync('result', 'utf8'); }
export function pureAsserted() { equal(1, 1); }
export function pureUnasserted() { return 1; }

export function namespaceAsserted() { equal(fsNs.readFileSync('a', 'utf8'), 'x'); }
export function namespaceUnasserted() { fsNs.readFileSync('a', 'utf8'); }
export function defaultAsserted() { equal(fsDefault.readFileSync('a', 'utf8'), 'x'); }
export function defaultUnasserted() { fsDefault.readFileSync('a', 'utf8'); }
export function bareDefaultAsserted() { equal(bareFs.existsSync('a'), true); }
export function bareDefaultUnasserted() { bareFs.existsSync('a'); }
export function defaultPromisesAsserted() { equal(fsDefault.promises.readFile('a'), 1); }
export function defaultPromisesUnasserted() { bareFs.promises.readFile('a'); }
export async function promisesAsserted() { equal(await readFile('a', 'utf8'), 'x'); }
export async function promisesUnasserted() { await bareRead('a', 'utf8'); }

// Negative controls: shapes outside the model stay unanalyzable even when asserted.
// (aliasedDefault is followed by the canonical alias graph, so it is modeled and passes.)
export function computedMember(name: 'readFileSync') { equal(fsDefault[name]('a', 'utf8'), 'x'); }
export function aliasedDefault() { const f = fsDefault; equal(f.readFileSync('a', 'utf8'), 'x'); }
export function otherModuleAsserted() { equal(hostname(), 'h'); }
export function dynamicRequire() { equal(require('node:fs').readFileSync('a', 'utf8'), 'x'); }
// An fs callback or promise continuation has no resolved invocation: uncertain.
export function callbackAsserted() { readFile('a', 'utf8').then((text) => { equal(text, 'x'); }); }
export function writeOnly() { fsNs.writeFileSync('a', 'x'); equal(1, 1); }
// no-mistakes-disable-next-line coverage-producer-assertion-owner -- suppression still applies to fs producers
export function fsSuppressed() { readFileSync('a', 'utf8'); }
// A callback parameter handed to a callback-style fs API is not a resolved invocation.
export function paramCallback(done: () => void) { fsNs.readFile('a', done); equal(1, 1); }
// A dead fs call (after return) is not executed, so it cannot make the producer uncertain.
export function deadFs() { equal(1, 1); return; fsNs.readFile('a', () => {}); }
export function ternaryCallback(flag: boolean, first: () => void, second: () => void) {
  fsNs.readFile('a', flag ? first : second);
}
export function logicalCallback(first: (() => void) | undefined, second: () => void) {
  fsNs.readFile('a', first || second);
}
export function computedCallback() { fsNs.readFile('a', makeCallback()); }
function makeCallback(): () => void { return () => {}; }
export function literalLastArgument() { fsNs.readFile('a', 'utf8'); }
