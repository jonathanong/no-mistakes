import { query, sql } from '@example/db';
declare const source: any;
declare const provider: any;
declare const iterable: string[];
declare const holder: any;
declare const extra: unknown;
declare let counter: number;
declare const object: { value: unknown };

// Each rejected shape would otherwise turn a runtime table name into finite SQL.
export function destructuredSet(table: string) {
  const { allowed } = source;
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* destructuredSet */ SELECT 1 FROM `.append(table));
}
export function memberConstructor(table: string) {
  const allowed = new provider.Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* memberConstructor */ SELECT 1 FROM `.append(table));
}
export function extraConstructorArgument(table: string) {
  const allowed = new Set(['safe_one'], extra);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* extraConstructorArgument */ SELECT 1 FROM `.append(table));
}
export function runtimeIterable(table: string) {
  const allowed = new Set(iterable);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* runtimeIterable */ SELECT 1 FROM `.append(table));
}
export function emptySet(table: string) {
  const allowed = new Set([]);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* emptySet */ SELECT 1 FROM `.append(table));
}
export function numericElement(table: string) {
  const allowed = new Set([1]);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* numericElement */ SELECT 1 FROM `.append(table));
}
export function positiveCondition(table: string) {
  const allowed = new Set(['safe_one']);
  if (allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* positiveCondition */ SELECT 1 FROM `.append(table));
}
export function elseCondition(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe'); else source.touch();
  return query(sql`/* elseCondition */ SELECT 1 FROM `.append(table));
}
export function memberReceiver(table: string) {
  const allowed = new Set(['safe_one']);
  if (!holder.allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* memberReceiver */ SELECT 1 FROM `.append(table));
}
export function memberArgument(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table.trim())) throw new Error('unsafe');
  return query(sql`/* memberArgument */ SELECT 1 FROM `.append(table));
}
export function updateSink(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* updateSink */ SELECT ${counter++}`.append(table));
}
export function deleteSink(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* deleteSink */ SELECT ${delete object.value}`.append(table));
}
export function arrowSink(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* arrowSink */ SELECT ${(() => table)()}`.append(table));
}
export function functionSink(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* functionSink */ SELECT ${(function () { return table })()}`.append(table));
}
export function classSink(table: string) {
  const allowed = new Set(['safe_one']);
  if (!allowed.has(table)) throw new Error('unsafe');
  return query(sql`/* classSink */ SELECT ${class Nested {}}`.append(table));
}
