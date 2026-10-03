import { query } from '@example/db';

// Statement-start directives also cover SQL whose relation is on a later physical line.
// no-mistakes-disable-next-line postgres-bounded-statements
await query(`SELECT id
FROM invoices`);
await query('SELECT id' + // no-mistakes-disable-line postgres-bounded-statements
  '\nFROM invoices');
// no-mistakes-disable-next-line postgres-bounded-statements
await query(`UPDATE
exports SET s3_key = NULL`);
await query('UPDATE' + // no-mistakes-disable-line postgres-bounded-statements
  '\nexports SET s3_key = NULL');
// no-mistakes-disable-next-line postgres-bounded-statements
await query(`DELETE
FROM sessions`);
await query('DELETE' + // no-mistakes-disable-line postgres-bounded-statements
  '\nFROM sessions');
await query(`SELECT id
FROM invoices`);
