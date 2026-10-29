import { pool } from '@example/db';
const sql = 'DROP TABLE users';
pool.read.call(pool, sql);
const { write: put } = pool;
put.apply(pool, [sql]);
consume(pool.execute);
const run = pool.execute;
run(sql);
function custom({ execute } = pool) {
  execute(sql);
}
custom();
