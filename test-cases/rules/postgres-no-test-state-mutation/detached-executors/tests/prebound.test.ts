import { pool } from '@example/db';
// The invocation's SELECT is not the SQL already bound into the executor.
const prebound = pool.query.bind(pool, 'DROP TABLE prebound_ddl');
const alias = prebound;
alias('SELECT 1');
