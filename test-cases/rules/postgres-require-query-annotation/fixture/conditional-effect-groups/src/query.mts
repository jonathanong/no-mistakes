import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { add, mutating } from './helpers.mjs';

function good(tail) {
  const q = sql`/* good */ SELECT 1`;
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  read(q); // accepted
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
}

function bad(tail) {
  const q = sql`/* bad */ SELECT 1`;
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  read(q); // unanalyzable
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  add(q, tail);
  mutating(q, tail);
}
