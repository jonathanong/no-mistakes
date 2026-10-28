import sql from 'sql-template-strings';
import { read } from '@bench/db';

export function unsafeUses(tail) {
  // Each rejected use could expose or replace the builder outside the closed set.
  const escaped = sql`/* escaped */ SELECT 1`;
  const append = () => { escaped.append(tail); };
  opaque(escaped);
  read(escaped); // low: false
  const used = sql`/* used */ SELECT 1`;
  const alias = () => used.append(tail);
  read(used); // low: false
  const stored = sql`/* stored */ SELECT 1`;
  const hold = () => { globalThis.value = stored; };
  read(stored); // low: false
  const defaulted = sql`/* defaulted */ SELECT 1`;
  function callback(x = defaulted) { return x; }
  read(defaulted); // low: false
  const mutated = sql`/* mutated */ SELECT 1`;
  const mutate = () => { mutated.text = 'DROP TABLE x'; };
  read(mutated); // low: false
  const other = sql`/* other */ SELECT 1`;
  const suspicious = () => other.values;
  read(other); // low: false
  void append; void alias; void hold; void callback; void mutate; void suspicious;
}
