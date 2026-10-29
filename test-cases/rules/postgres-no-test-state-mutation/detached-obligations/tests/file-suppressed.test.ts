// no-mistakes-disable-file postgres-no-test-state-mutation
import { pool } from '@example/db';
consume(pool.query);
