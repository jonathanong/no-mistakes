import { owner } from './owner.mjs';
export function arrayImported() { [1].forEach(owner); }
export function promiseImported() { return Promise.resolve(1).finally(owner); }
export function arrayInline() { [1].forEach(() => owner()); }
function forward(callback: () => void) { return Promise.resolve(1).finally(callback); }
export function callerProven() { return forward(owner); }
export function unknown(callback: () => void) { return Promise.resolve(1).finally(callback); }
