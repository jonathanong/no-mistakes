import { assertion } from './assertion-namespace.mjs';
import { AsyncLocalStorage as Storage } from 'node:async_hooks';
import { strictEqual as assertValue } from 'node:assert';
import { owner } from './owner.mjs';
const storage = new Storage<string>();
export function leaves(rows: number[], text: string, n: number, expectation: { function: string }) {
  rows.map(r => r.toFixed(1)).join(',');
  rows.forEach(r => { r.toFixed(1); });
  rows.forEach(function (r) { r.toFixed(1); });
  text.trim().padStart(2).slice(1); n.toString();
  expectation.function.trim();
  const dayAnchorMs = Date.UTC(2025, 0, 1);
  dayAnchorMs.toString().padStart(2, '0').slice(0);
}
export function arrayImported(rows: number[]) { rows.forEach(owner); }
export function promiseImported() { return Promise.resolve(1).then(owner).finally(owner); }
export function boundAlias() { const callback = owner; return Promise.resolve(1).finally(callback); }
export function promiseInline() { return Promise.resolve(1).finally(() => owner()); }
export function storageImported() { return storage.run('id', owner); }
export function directAssertion() { return Promise.resolve(1).finally(assertValue); }
function arrayForward(fn: () => void) { [1].map(fn); }
function promiseForward(fn: () => void) { return Promise.resolve(1).then(fn); }
function storageForward(fn: () => void) { return storage.run('id', fn); }
export function callerArray() { arrayForward(owner); }
export function callerPromise() { return promiseForward(owner); }
export function callerStorage() { return storageForward(owner); }
export function unresolved(fn: () => void) { return Promise.resolve(1).finally(fn); }
export function unresolvedCall(fn: () => void) { fn(); }
export function storedMap() { const handlers = new Map(); handlers.set('check', owner); const fn = handlers.get('check'); fn(); }
export function storedArray() { const checks = [owner]; checks.forEach(check => check()); }
export function unknownReceiver(receiver: { finally: (fn: () => void) => void }) { receiver.finally(owner); }
function register(fn: () => void) { return 1; }
export function collision() {
  { const fn = owner; register(fn); }
  { const fn = 0; Promise.resolve(1).finally(fn); }
}
export function registered() { register(owner); }
function invoke(fn: () => void) { fn(); }
export function callerInvoke() { invoke(owner); }
function noop() { return 1; }
export function callbackNoOwner() { return Promise.resolve(1).finally(noop); }
export function namedLiteral() { return Promise.resolve(1).finally(function callback() { owner(); }); }
const verify = assertion.strictEqual;
export function namespaceAlias() { return Promise.resolve(1).finally(verify); }
export function spreadContinuation(callbacks: (() => void)[]) { return Promise.resolve(1).then(...callbacks, owner); }
export function spreadAfterAssertion(callbacks: (() => void)[]) { return Promise.resolve(1).then(owner, ...callbacks); }
export function stringArray() { return 'a,b'.split(',').forEach(owner); }
export function unionRows(unionValues: string[] | number[]) { unionValues.forEach(owner); }
export function numberRows(mixedRows: number[]) { mixedRows.forEach(owner); }
export function textRows(mixedRows: string[]) { mixedRows.forEach(owner); }
export function emptyNumbers() { const emptyMixed: number[] = []; emptyMixed.forEach(owner); }
export function emptyTexts() { const emptyMixed: string[] = []; emptyMixed.forEach(owner); }
export function foreignRun() { return ({}).run('id', owner); }
