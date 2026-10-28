import sql from 'sql-template-strings';
import { read } from '@bench/db';

function add(q, tail) { q.append(tail); }
function twice(q, tail) { q.append(tail).append(' '); }
function usedResult(q, tail) { const alias = q.append(tail); opaque(alias); }
function returnedResult(q, tail) { return q.append(tail); }
function passedResult(q, tail) { opaque(q.append(tail)); }
function defaultEscape(q, other = opaque(q)) { q.append(other); }
function selfSuffix(q) { q.append(opaque(q)); }
function captured(q, tail) { return () => { q.append(tail); }; }
function forwarded(q) { opaque(q); }
function optional(q, tail) { q?.append(tail); }
function spread(q, tail) { q.append(...tail); }
// The implicit argument array can leak q without a direct parameter reference.
function argumentsAlias(q, tail) { const alias = arguments[0]; alias.append(tail); }
// Even a noncall eval reference conservatively blocks this strict summary.
function evalReference(q, tail) { opaque(eval); q.append(tail); }

function safe(tail) {
  const q = sql`/* safe */ SELECT 1`;
  add(q, tail);
  twice(q, tail);
  read(q); // low: true
  read(q.append(tail)); // low: true
}
function unsafe(tail) {
  const a = sql`/* a */ SELECT 1`;
  usedResult(a, tail);
  read(a); // low: false
  const b = sql`/* b */ SELECT 1`;
  returnedResult(b, tail);
  read(b); // low: false
  const c = sql`/* c */ SELECT 1`;
  passedResult(c, tail);
  read(c); // low: false
  const d = sql`/* d */ SELECT 1`;
  defaultEscape(d);
  read(d); // low: false
  const e = sql`/* e */ SELECT 1`;
  selfSuffix(e);
  read(e); // low: false
  const f = sql`/* f */ SELECT 1`;
  captured(f, tail);
  read(f); // low: false
  const g = sql`/* g */ SELECT 1`;
  forwarded(g);
  read(g); // low: false
  const h = sql`/* h */ SELECT 1`;
  optional(h, tail);
  read(h); // low: false
  const i = sql`/* i */ SELECT 1`;
  spread(i, tail);
  read(i); // low: false
  const j = sql`/* j */ SELECT 1`;
  const alias = add(j, tail);
  read(j); // low: false
}
function order(tail) {
  const q = sql`/* order */ SELECT 1`;
  read(q); // low: true
  forwarded(q);
  read(q); // low: false
}
function implicitAliases(tail) {
  const a = sql`/* arguments */ SELECT 1`;
  argumentsAlias(a, tail);
  read(a); // low: false
  const b = sql`/* eval */ SELECT 1`;
  evalReference(b, tail);
  read(b); // low: false
}
