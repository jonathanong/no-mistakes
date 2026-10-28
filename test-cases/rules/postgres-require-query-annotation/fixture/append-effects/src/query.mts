import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { add, addMore, returned, composed, aliased, defaulted, captured, forwarded } from './helpers.mjs';

function safe(tail) {
  const q = sql`/* imported */ SELECT 1`;
  add(q, tail);
  addMore(q, tail);
  composed(q, tail);
  read(q); // accepted
  read(q.append(tail)); // accepted
}
function discardedReturnedEffect(tail) {
  const q = sql`/* unknown */ SELECT 1`;
  add(q, tail);
  returned(q, tail);
  read(q); // accepted
}
function usedResult(tail) {
  const q = sql`/* result */ SELECT 1`;
  const alias = add(q, tail);
  read(q); // unanalyzable
}
function escape(tail) {
  const a = sql`/* a */ SELECT 1`;
  aliased(a, tail);
  read(a); // unanalyzable
  const b = sql`/* b */ SELECT 1`;
  defaulted(b);
  read(b); // unanalyzable
  const c = sql`/* c */ SELECT 1`;
  captured(c, tail);
  read(c); // unanalyzable
  const d = sql`/* d */ SELECT 1`;
  forwarded(d);
  read(d); // unanalyzable
}
function order(tail) {
  const q = sql`/* order */ SELECT 1`;
  read(q); // accepted
  add(q, tail);
  read(q); // accepted
  forwarded(q);
  read(q); // unanalyzable
}
function loop(tail) {
  const q = sql`/* loop */ SELECT 1`;
  for (const value of tail) {
    add(q, value);
    read(q); // accepted
  }
}
