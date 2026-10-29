import { pool } from '@example/db';
const { query: q } = pool;
type Q = typeof q;
export type { Q, q };
export { type q as QueryType };
function local(pool: { query: Function }, { query } = pool) {
  consume(query);
}
