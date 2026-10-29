import { pool } from '@example/db';
// no-mistakes-disable-next-line postgres-no-test-state-mutation
consume(pool.query);
consume(pool.query); // no-mistakes-disable-line postgres-no-test-state-mutation
