import { pool } from '@example/db';
// Copying the function before the later property write keeps MAX's original capture event.
const { query } = pool;
pool.query = replacement;
const first = query;
const second = first;
second('DROP TABLE preserved_copy');
