import { strictEqual } from 'node:assert';
import { withOwner } from './wrapper.mjs';
function owner() { strictEqual(1, 1); return true; }
export function checked() { return withOwner(owner); }
export function unknown(handler: () => boolean) { return withOwner(handler); }
import { withNoOwner } from './wrapper.mjs';
function secondOwner() { strictEqual(2, 2); return false; }
function noop() { return false; }
export function checkedSecond() { return withOwner(secondOwner); }
export function noOwner() { return withNoOwner(noop); }
export function missingArgument() { return withOwner(); }
import * as namespaceValue from 'node:assert';
import { withNamespaceOnly } from './wrapper.mjs';
export function namespaceInput() { return withOwner(namespaceValue); }
export function onlyNamespace() { return withNamespaceOnly(namespaceValue); }
export function unrelated(callback: () => void) { callback(); callback(); callback(); callback(); }
