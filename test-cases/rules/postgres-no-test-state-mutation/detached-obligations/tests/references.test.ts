import { pool } from '@example/db';
const sql = 'DROP TABLE users';
pool.query.bind(pool)(sql);
const { query: q } = pool;
q.call(pool, sql);
q.apply(pool, [sql]);
Reflect.apply(pool.query, pool, [sql]);
consume(pool.query);
consume(q);
export { q };
function f({ query } = pool) {
  query(sql);
}
function g(text: string, { query: run } = pool) {
  run(text);
}
f();
g(sql);
const alias = q;
consume(alias);
const unused = pool.query;
const exact = pool.query;
exact(sql);
function unusedDefault({ query } = pool) {}
