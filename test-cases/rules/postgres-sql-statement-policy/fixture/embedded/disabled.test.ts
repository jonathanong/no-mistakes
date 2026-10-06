// no-mistakes-disable-file postgres-sql-statement-policy
import { query, sql } from '@example/db';
query(sql`CREATE TABLE disabled (id int)`);
