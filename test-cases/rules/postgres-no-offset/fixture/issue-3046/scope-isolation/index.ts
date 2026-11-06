import { query, Pool, createPool } from 'pg';
namespace First {
  type Pool = {};
  const local: Pool = {};
  var query = (sql: string) => {};
  query('SELECT id FROM posts OFFSET 90');
}
namespace Second {
  var sql = 'SELECT id FROM posts OFFSET 5';
  query(sql);
  const pool = createPool();
  pool.query('SELECT id FROM posts OFFSET 6');
}
namespace Third {
  const pool = { query(sql: string) {} };
  pool.query('SELECT id FROM posts OFFSET 91');
}
const pool: Pool = createPool();
pool.query('SELECT id FROM posts OFFSET 7');
query('SELECT id FROM posts OFFSET 8');
