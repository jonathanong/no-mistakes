import { query, sql } from '@example/db';
query(sql`ALTER TABLE orders ADD COLUMN excluded text`);
