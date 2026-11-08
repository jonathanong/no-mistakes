import { query } from '@example/db';
// no-mistakes-disable-next-line postgres-lock-written-rows-only: intentional exception
query(`
SELECT pg_advisory_xact_lock(1)
`, []);
