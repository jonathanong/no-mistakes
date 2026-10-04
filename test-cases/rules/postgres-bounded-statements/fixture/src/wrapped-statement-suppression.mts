import { query } from '@example/db';

// The wrapper operand owns the directive even when the write is another operand.
// no-mistakes-disable-next-line postgres-bounded-statements
await query('EXPLAIN ANALYZE' + '\nUPDATE accounts SET email = $1');
await query('WITH chosen AS (SELECT 1)' + // no-mistakes-disable-line postgres-bounded-statements
  '\nDELETE FROM accounts');
await query(`EXPLAIN ANALYZE
UPDATE accounts SET email = $1`);
await query(`WITH chosen AS (SELECT 1)
DELETE FROM accounts`);
