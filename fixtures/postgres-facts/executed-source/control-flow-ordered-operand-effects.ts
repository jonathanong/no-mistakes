import { read, sql } from '@example/db';
export const sharedQuery = sql`SELECT 1`;

export function callOperands(object: unknown, opaque: unknown) {
  opaque(object.mutate, read(sharedQuery));
}

export function branchArray(object: unknown, flag: boolean) {
  if (flag) {
    [object.mutate, read(sharedQuery)];
  }
}

export function branchObject(object: unknown, flag: boolean) {
  if (flag) {
    ({ first: object.mutate, second: read(sharedQuery) });
  }
}

export function binaryOperands(object: unknown) {
  const ignored = object.mutate + read(sharedQuery);
}

export function pureArray() {
  [0, read(sharedQuery)];
}

export function pureObject() {
  ({ first: 0, second: read(sharedQuery) });
}

export function sequenceOperands(object: unknown) {
  (object.mutate, read(sharedQuery));
}

export function appendOperands(object: unknown) {
  (object.mutate, sharedQuery).append(read(sharedQuery));
}

export function taggedOperands(object: unknown) {
  sql`${object.mutate}${read(sharedQuery)}`;
}

export function opaqueOperands(object: unknown, Constructor: unknown) {
  new Constructor(object.mutate, read(sharedQuery));
}

export function templateOperands(object: unknown) {
  `${object.mutate}${read(sharedQuery)}`;
}

export function calleeOperands(object: unknown, opaque: unknown) {
  (object.mutate, opaque)(read(sharedQuery));
}

export function earlierValidRead(object: unknown) {
  [read(sharedQuery), object.mutate];
}

export function branchTestAfterEffect(object: unknown) {
  if ((object.mutate, read(sharedQuery), true)) {}
}

export function branchTestEarlierRead(object: unknown) {
  if ((read(sharedQuery), object.mutate, true)) {}
}

export function computedReceiver(object: unknown, flag: boolean) {
  if (flag) object.mutate[read(sharedQuery)];
}

export function deleteReceiver(object: unknown) {
  delete object.mutate[read(sharedQuery)];
}

export function computedEarlierRead(object: unknown) {
  read(sharedQuery)[object.mutate];
}

export function computedPure() {
  const object = { first: 0 };
  object['first' + read(sharedQuery)];
}

export function methodGetter(object: unknown) {
  object.method(read(sharedQuery));
}

export function computedMethodGetter(object: unknown) {
  object['method'](read(sharedQuery));
}

export function indexedMethodGetter(object: unknown) {
  object[0](read(sharedQuery));
}

export function compoundGetter(object: unknown) {
  const ignored = (object.counter += read(sharedQuery));
}

export function simpleAssignment(object: unknown) {
  const ignored = (object.counter = read(sharedQuery));
}

export function computedCompound(object: unknown) {
  const ignored = (object[read(sharedQuery)] +=
    read(sharedQuery));
}

export function recordMethod() {
  const object = { method: (value: unknown) => value };
  object.method(read(sharedQuery));
}

export function arrayMethod() {
  [function (value: unknown) { return value; }][0](read(sharedQuery));
}

export function unsupportedMethodShorthand() {
  // Method shorthand is still outside the supported record-data projection.
  const object = { method(value: unknown) { return value; } };
  object.method(read(sharedQuery));
}

export function bindingCompound() {
  let counter = 0;
  const ignored = (counter += read(sharedQuery));
}
