// no-mistakes-disable-file postgres-no-test-state-mutation
import { pool } from '@example/db';
pool.query.call(pool, "SELECT 1");
