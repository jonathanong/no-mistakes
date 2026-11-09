import { strictEqual } from 'node:assert';
import { owner } from './owner.mjs';

export function reviverOwner() { JSON.parse('{"value":1}', owner); }
export function inlineReviverOwner() {
  JSON.parse('{"value":1}', function reviver() { strictEqual(1, 1); return 1; });
}
export function reviverNoOwner() { JSON.parse('{"value":1}', () => 1); }
export function absentReviver() { JSON.parse('{"value":1}'); }
export function literalReviver() { JSON.parse('{"value":1}', null); }
export function unknownReviver(fn: (key: string, value: unknown) => unknown) {
  JSON.parse('{"value":1}', fn);
}
export function stringifyOwner() { JSON.stringify({ value: 1 }, owner); }
export function stringifyUnknown(fn: (key: string, value: unknown) => unknown) {
  JSON.stringify({ value: 1 }, fn);
}
export function stringifyArray() { JSON.stringify({ value: 1 }, ['value']); }
export function stringifyAbsent() { JSON.stringify({ value: 1 }); }
