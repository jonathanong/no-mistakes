import { pool } from '@example/db';
const { query } = pool;
// no-mistakes-disable-next-line postgres-no-test-state-mutation -- saved suppression control
query('DROP TABLE next_line_suppressed');
query('DROP TABLE same_line_suppressed'); // no-mistakes-disable-line postgres-no-test-state-mutation -- saved suppression control
query('DROP TABLE unsuppressed_control');
