import { read, sql } from '@example/db';
const sharedQuery = sql`SELECT 1`;

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
