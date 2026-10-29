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
export function templateLiteralArgument() { fsNs.readFile('a', `utf8`); }
export function typedLiteralArgument() { fsNs.readFile('a', 'utf8' as const); }
export function dynamicTemplateArgument(encoding: string) { fsNs.readFile('a', `${encoding}`); }
export function spreadArguments(...callbacks: Array<() => void>) { fsNs.readFile('a', ...callbacks); }
export function noArguments() { fsNs.readFile(); }
export function applyLiteralArgument() { fsNs.watch.apply(null, ['file']); }
export function applyTemplateLiteralArgument() { fsNs.watch.apply(null, [`file`]); }
export function applyCallbackArgument(flag: boolean, first: () => void, second: () => void) { fsNs.watch.apply(null, ['file', flag ? first : second]); }
export function applyNoArguments() { fsNs.watch.apply(null, []); }
export function applyNullArguments() { fsNs.watch.apply(null, null); }
export function applySpreadArguments(...callbacks: Array<() => void>) { fsNs.watch.apply(null, ['file', ...callbacks]); }
export function callLiteralArgument() { fsNs.watch.call(null, 'file'); }
export function callCallbackArgument(first: (() => void) | undefined, second: () => void) { fsNs.watch.call(null, 'file', first || second); }
export function streamOptions() { equal(fsNs.createReadStream('a', { encoding: 'utf8' }).path, 'a'); }
export function mkdirOptions() { equal(fsNs.mkdir('dir', { recursive: true }), undefined); }
export function typedStreamOptions() { equal(fsNs.createReadStream('a', { encoding: 'utf8' } as const).path, 'a'); }
export function arrayOptions() { equal(typeof fsNs.watch('a', []), 'object'); }
export function callObjectOptions() { equal(fsNs.createReadStream.call(null, 'a', { encoding: 'utf8' }).path, 'a'); }
export function applyObjectOptions() { equal(fsNs.createReadStream.apply(null, ['a', { encoding: 'utf8' }]).path, 'a'); }
export function applyArrayOptions() { equal(typeof fsNs.watch.apply(null, ['a', []]), 'object'); }
export function ternaryOptions(flag: boolean) { equal(fsNs.createReadStream('a', flag ? { encoding: 'utf8' } : { encoding: 'ascii' }).path, 'a'); }
