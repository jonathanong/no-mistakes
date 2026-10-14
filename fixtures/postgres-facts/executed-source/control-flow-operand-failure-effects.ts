import { read, sql } from '@example/db';

export function indexedReceiver(object: unknown) {
  const query = sql`SELECT 1`;
  const parts = ['a'];
  (object.mutate, parts)[0] = 'x';
  read(query);
}

export function binaryLeft(object: unknown) {
  const query = sql`SELECT 1`;
  const ignored = object.mutate + '';
  read(query);
}

export function binaryRight(object: unknown) {
  const query = sql`SELECT 1`;
  const ignored = '' + object.mutate;
  read(query);
}

export function indexedPure() {
  const query = sql`SELECT 1`;
  const parts = ['a'];
  (0, parts)[0] = 'x';
  read(query);
}

export function binaryPure() {
  const query = sql`SELECT 1`;
  const ignored = 'a' + 'b';
  read(query);
}

export function indexedRhs(object: unknown) {
  const query = sql`SELECT 1`;
  const parts = ['a'];
  parts[0] = (object.mutate, 'x');
  read(query);
}

const sharedQuery = sql`SELECT 1`;
function returnAccessor(object: unknown) {
  return object.mutate;
}
export function returnedFailure(object: unknown) {
  returnAccessor(object);
  read(sharedQuery);
}
