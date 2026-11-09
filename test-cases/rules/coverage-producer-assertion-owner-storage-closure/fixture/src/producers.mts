import { AsyncLocalStorage } from 'node:async_hooks';
import { strictEqual } from 'node:assert';
const storage = new AsyncLocalStorage<string>();
function owner() { strictEqual(1, 1); }
function bridge(fn: () => void) { fn(); }
function delegate(next: (fn: () => void) => void, fn: () => void) { next(fn); }
export function ambiguous() { storage.run('id', delegate, bridge, owner); }
function missingBridge(fn: () => void) { fn(); }
function missingDelegate(fn: () => void) { missingBridge(fn); }
export function missing() { storage.run('id', missingDelegate, owner); }
export function ordinaryMissing() { missingDelegate(); }
export function unrelated(fn: () => void) { fn(); fn(); fn(); fn(); }
function directBridge(fn: () => void) { fn(); }
function directDelegate(fn: () => void) { directBridge(fn); }
export function forwarded() { storage.run('id', directDelegate, owner); }
