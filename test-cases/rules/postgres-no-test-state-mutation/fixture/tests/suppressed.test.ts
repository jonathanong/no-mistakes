import { query } from '@example/db';
// Each suppression protects a separate executor call, not inert SQL text.
// no-mistakes-disable-next-line postgres-no-test-state-mutation
query('DROP TABLE next_line');
query('DROP TABLE same_line'); // no-mistakes-disable-line postgres-no-test-state-mutation
