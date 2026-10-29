// no-mistakes-disable-file postgres-no-test-state-mutation -- saved suppression control
import { pool } from '@example/db';
const bound = pool.query.bind(pool);
bound('DROP TABLE file_suppressed');
