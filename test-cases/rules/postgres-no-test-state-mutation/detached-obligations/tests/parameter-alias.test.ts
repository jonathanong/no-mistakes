import { pool } from '@example/db';
function f(client = pool, { query: run } = client) {
  run('DROP TABLE parameter_receiver');
}
f();
function g(run = pool.execute) {
  run('DROP TABLE parameter_executor');
}
g();
