import { pool } from '@example/db';
function exact() {
  const { query } = pool;
  const alias = query;
  alias('DROP TABLE local_detached_body');
}
exact();
function uncertain() {
  let query;
  query = pool.query.bind(pool);
  query('DROP TABLE local_uncertain_body');
}
uncertain();
