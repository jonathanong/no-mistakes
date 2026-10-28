import sql from 'sql-template-strings';
import { read } from '@bench/db';

export async function arbitraryOrder(items, tail) {
  const q = sql`/* captured_order */ SELECT 1`;
  // Either closure may run first; every permitted operation preserves the named head.
  const appendLater = async () => { q.append(tail); };
  const readFirst = async () => { read(q); }; // low: true
  await Promise.all([readFirst(), appendLater()]);
  await appendLater();
  read(q); // low: true
  items.forEach(() => { q.append(tail); });
  read(q); // low: true
}
