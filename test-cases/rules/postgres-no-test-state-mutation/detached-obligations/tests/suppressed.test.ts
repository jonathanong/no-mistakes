import { pool } from '@example/db';
// no-mistakes-disable-next-line postgres-no-test-state-mutation
pool.query.call(pool, "SELECT 1");
pool.query.call(pool, "SELECT 1"); // no-mistakes-disable-line postgres-no-test-state-mutation
