import sql from 'sql-template-strings';
import { read } from '@bench/db';

export function shadowed(tail) {
  const q = sql`/* outer */ SELECT 1`;
  const append = () => { q.append(tail); };
  const inner = () => {
    const q = 42;
    read(q); // low: false
  };
  read(q); // low: true
  void append; void inner;
}
