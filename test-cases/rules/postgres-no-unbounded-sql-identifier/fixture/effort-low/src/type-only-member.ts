import type { pool as erasedPool } from '@example/db';

// Deliberately invalid runtime use: an erased import cannot select an executor.
erasedPool.query('DROP TABLE app.erased_import');
