import { read, sql } from '@example/db';
export function boundAccessorBeforeAllocation(object: unknown) {
  // An opaque getter or Proxy can mutate intrinsics before any builder exists.
  const ignored = object.mutateBuiltins;
  read(['SELECT 1'].join(''));
}
export function opaqueConstructorBeforeAllocation(Constructor: unknown) {
  new Constructor();
  read(['SELECT 1'].join(''));
}
export function opaqueTagBeforeAllocation(tag: unknown) {
  tag`ignored`;
  read(['SELECT 1'].join(''));
}
export function taggedGetterBeforeAllocation(object: unknown) {
  sql`ignored ${object.value}`;
  read(['SELECT 1'].join(''));
}
export function opaqueSetterBeforeAllocation(object: unknown) {
  object[0] = 'x';
  read(['SELECT 1'].join(''));
}
export function opaqueDeleteBeforeAllocation(object: unknown) {
  delete object.value;
  read(['SELECT 1'].join(''));
}
export function opaqueArgumentDeleteKey(key: unknown) {
  delete arguments[key];
  read(['SELECT 1'].join(''));
}
function knownArgumentDelete() {
  const primitive = [].forEach(() => 0);
  String.raw`${primitive}`;
  delete arguments[0];
  read(['/* intrinsics/known-delete */ SELECT 1'].join(''));
}
export function knownArgumentDeleteCaller() { knownArgumentDelete(); }
function knownArgumentDeleteKey() {
  const key = '0';
  delete arguments[key];
  const primitiveKey = [].forEach(() => 0);
  delete arguments[primitiveKey];
  return ['/* intrinsics/known-delete-key */ SELECT 1'].join('');
}
export function knownArgumentDeleteKeyCaller() { read(knownArgumentDeleteKey()); }
export function opaqueRawTagCoercion(object: unknown) {
  String.raw`${object}`;
  read(['SELECT 1'].join(''));
}
