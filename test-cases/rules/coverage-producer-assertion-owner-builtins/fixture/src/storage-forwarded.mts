import { AsyncLocalStorage } from 'node:async_hooks';
import { owner } from './owner.mjs';
const storage = new AsyncLocalStorage<string>();
function invokeRun(fn: () => void) { fn(); }
function invokeExit(fn: () => void) { fn(); }
function conflicting(fn: () => void) { fn(); }
export function forwardedRun() { storage.run('id', invokeRun, owner); }
export function forwardedExit() { storage.exit(invokeExit, owner); }
export function forwardedConflict() { storage.run('id', conflicting, owner); }
export function unknownInput(fn: () => void) { conflicting(fn); }
function invokeSpread(fn: () => void) { fn(); }
export function forwardedSpread(callbacks: (() => void)[]) {
  storage.run('id', invokeSpread, ...callbacks, owner);
}
function invokeMissing(fn: () => void) { fn(); }
export function missingAlternate() {
  storage.run('id', invokeMissing, owner);
  storage.run('id', invokeMissing);
}
function invokePromise(fn: () => void) { fn(); }
export function promiseAlternate() {
  storage.run('id', invokePromise, owner);
  Promise.resolve().then(invokePromise);
}
function invokeArray(fn: () => void) { fn(); }
export function arrayAlternate() {
  storage.run('id', invokeArray, owner);
  [1].forEach(invokeArray);
}
function invokeTimer(fn: () => void) { fn(); }
export function timerAlternate() {
  storage.run('id', invokeTimer, owner);
  setTimeout(invokeTimer);
}
function ordinaryArrayInvoke(fn: () => void) { fn(); }
export function ordinaryArrayOwner() { ordinaryArrayInvoke(owner); }
export function ordinaryArrayAlternate() { [1].map(ordinaryArrayInvoke); }
function ordinaryPromiseInvoke(fn: () => void) { fn(); }
export function ordinaryPromiseOwner() { ordinaryPromiseInvoke(owner); }
export function ordinaryPromiseAlternate() { Promise.resolve().then(ordinaryPromiseInvoke); }
function ordinaryTimerInvoke(fn: () => void) { fn(); }
export function ordinaryTimerOwner() { ordinaryTimerInvoke(owner); }
export function ordinaryTimerAlternate() { setTimeout(ordinaryTimerInvoke); }
