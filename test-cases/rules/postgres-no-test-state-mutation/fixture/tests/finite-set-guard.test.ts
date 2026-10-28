import { query, sql } from '@example/db';
declare function consume(value: unknown): void;
declare const names: string[];

export function guardedRead(table: string) {
  const allowed = new Set(['safe_one', 'safe_two', 'safe_three']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* guardedRead */ SELECT 1 FROM `.append(table));
}

export function guardedDdl(table: string) {
  const allowed = new Set(['TABLE first', 'TABLE second']);
  if (!allowed.has(table)) { throw new Error('invalid table'); }
  return query(sql`/* guardedDdl */ DROP `.append(table));
}

export function guardedTemplate(table: string) {
  const allowed = new Set([`safe_one`, `safe_one`, `safe_two`]);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* guardedTemplate */ SELECT 1 FROM `.append(table));
}

export function typeOnly(table: 'safe_one' | 'safe_two') {
  return query(sql`/* typeOnly */ SELECT 1 FROM `.append(table));
}
export function wrongArgument(table: string, other: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(other)) throw new Error('invalid table');
  return query(sql`/* wrongArgument */ SELECT 1 FROM `.append(table));
}
export function noThrow(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) consume(table);
  return query(sql`/* noThrow */ SELECT 1 FROM `.append(table));
}
export function computedHas(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed['has'](table)) throw new Error('invalid table');
  return query(sql`/* computedHas */ SELECT 1 FROM `.append(table));
}
export function optionalHas(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has?.(table)) throw new Error('invalid table');
  return query(sql`/* optionalHas */ SELECT 1 FROM `.append(table));
}
export function shadowedSet(Set: any, table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* shadowedSet */ SELECT 1 FROM `.append(table));
}
export function spreadSet(table: string) {
  const allowed = new Set([...names]);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query(sql`/* spreadSet */ SELECT 1 FROM `.append(table));
}
export function escapedSet(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query((consume(allowed), sql`/* escapedSet */ SELECT 1 FROM `.append(table)));
}
export function aliasedSet(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  const alias = allowed;
  return query(sql`/* aliasedSet */ SELECT 1 FROM `.append(table));
}
export function mutatedSet(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query((allowed.add('other'), sql`/* mutatedSet */ SELECT 1 FROM `.append(table)));
}
export function reassignedArgument(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query((table = 'other', sql`/* reassignedArgument */ SELECT 1 FROM `.append(table)));
}
export function overwrittenHas(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('invalid table');
  return query((allowed.has = () => true, sql`/* overwrittenHas */ SELECT 1 FROM `.append(table)));
}
export function caughtThrow(table: string) {
  const allowed = new Set(['safe_one']);
  try { if (!allowed.has(table)) throw new Error('invalid table'); } catch {}
  return query(sql`/* caughtThrow */ SELECT 1 FROM `.append(table));
}
