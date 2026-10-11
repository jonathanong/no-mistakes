import { query, sql } from "@example/db";
query(sql`SET ${flag ? sql`
  work_mem = '4MB' -- findings: setting:work_mem
` : sql`statement_timeout = '4MB'`}`);
query(sql`SELECT set_config(${flag ? sql`
  'work_mem' -- findings: setting:work_mem
` : sql`'statement_timeout'`}, '4MB', true)`);
query(sql`${flag ? sql`
  ALTER TABLE accounts ADD COLUMN note text -- findings: ALTER TABLE
` : sql`SELECT 1`}`);
