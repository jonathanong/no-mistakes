import sql from 'sql-template-strings';
import { read } from '@bench/db';

export function gap(tail) {
  const q = sql`/* gap */ SELECT 1`;
  const append = () => { q.append(tail); };
  // Direct eval can access lexical bindings without a recorded q reference.
  eval('void 0');
  read(q); // low: false
  void append;
}
