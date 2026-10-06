import { query, sql } from '@example/db';
// Keep both routine projections: direct DDL and statically recoverable EXECUTE.
query(sql`DO $$ BEGIN
  CREATE TABLE direct_table (id int);
  EXECUTE 'ALTER TABLE orders ADD COLUMN routine_column text';
END $$`);
