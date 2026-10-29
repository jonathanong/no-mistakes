import { pool } from '@example/db';
// Max effort only: low effort deliberately reports every alias use (#2500).
const query = pool.query;
pool.query = query;
const original = pool.write;
if (pool.query !== query) {
  consume(original);
}
const chosen = pool.query ?? pool.write;
const picked = pool.query ? pool.write : pool.read;
const { query: detached, write } = pool;
export { chosen, picked, detached, write };
