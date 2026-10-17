import { read, sql } from '@example/db';

function appendKnown(query: unknown) {
  query.append(sql`; DELETE FROM records`);
}

export function appendReceiver() {
  const query = sql`SELECT 1`;
  const result = query.append((appendKnown(query), ''));
  read(query);
  read(result);
}

export function binaryLeft() {
  const query = sql`SELECT 1`;
  const text = query + (appendKnown(query), '');
  read(text);
}

export function appendPure() {
  const query = sql`SELECT 1`;
  const result = query.append('');
  read(result);
}

export function binaryPure() {
  const query = sql`SELECT 1`;
  const text = query + '';
  read(text);
}
