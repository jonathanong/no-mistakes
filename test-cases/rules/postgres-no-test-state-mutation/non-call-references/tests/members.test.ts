import { pool } from '@example/db';
// Every line mentions an executor without invoking it; none may report.
if (pool.query || pool.read) consume(1);
if (!pool.query) consume(2);
const { rows } = pool.query;
consume(pool.query as Function, { query: pool.query, vote: false });
consume(pool.query ? pool.write : pool.read, pool.query ?? pool.write);
const proxied = new Proxy(pool.write, {});
const same = pool.query === pool.write;
function withDefault({ query: run } = pool) {}
function withExecutor(run = pool.query) {}
export { rows, proxied, same };
