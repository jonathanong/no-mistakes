// no-mistakes-disable-file postgres-no-test-state-mutation
import { query } from '@example/db';
query('DROP TABLE entire_file');
