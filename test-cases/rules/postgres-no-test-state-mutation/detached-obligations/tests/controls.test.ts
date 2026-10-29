import { pool as other } from '@example/other';
import type { pool as typed } from '@example/db';
other.query.call(other, 'DROP TABLE users');
const local = { query() {} };
consume(local.query);
function shadow(pool: { query: Function }) {
  consume(pool.query);
}
consume(typed.query);
