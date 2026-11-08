import { AsyncLocalStorage } from 'node:async_hooks';
import { format } from './util.mjs';
import { owner } from './owner.mjs';
import * as Sentry from '@sentry/node';
const storage = new AsyncLocalStorage<string>();
export function builtinsOnly() { format([1, 2], 'a,b'); owner(); }
export function builtinsNoOwner() { format([1, 2], 'a,b'); }
export function callbackNamed(items: number[]) { items.map(owner); }
export function callbackInline(items: number[]) { items.forEach(() => owner()); }
export function callbackStorage() { storage.run('id', () => owner()); }
export function callbackStored(items: number[]) { format(items, 'a'); const stored = [() => owner()]; stored.length; }
export function callbackUnresolved(items: number[], fn: (value: number) => void) { format(items, 'a'); items.map(fn); }
export function thirdParty(error: Error) { format([1], 'a'); Sentry.captureException(error); }
export function many(a: () => void, b: () => void, c: () => void, d: () => void, e: () => void, f: () => void, g: () => void, h: () => void) { a(); b(); c(); d(); e(); f(); g(); h(); }
