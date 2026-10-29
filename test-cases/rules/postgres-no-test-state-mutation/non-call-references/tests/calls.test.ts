import { pool } from '@example/db';
// Positive controls: invoking an executor still reports.
const run = pool.execute;
run('DROP TABLE alias_control');
pool.query.call(pool, 'DROP TABLE call_control');
Reflect.apply(pool.write, pool, ['DROP TABLE reflect_control']);
