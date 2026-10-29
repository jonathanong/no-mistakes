import { pool } from '@example/db';
const { query } = pool;
query('DROP TABLE outer_before_shadow');
{
  const query = unrelatedFunction;
  query('DROP TABLE unrelated_shadow');
}
query('DROP TABLE outer_after_shadow');
