import { query, Pool, createPool } from 'pg';
namespace N {
  const sql = 'SELECT id FROM posts OFFSET 1';
  query(sql);
  const pool: Pool = createPool();
  pool.query('SELECT id FROM posts OFFSET 2');
}
