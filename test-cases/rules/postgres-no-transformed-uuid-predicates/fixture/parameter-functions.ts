import { query, sql } from "db";
declare const value: string;
query(sql`SELECT 1 FROM accounts WHERE id = uuid(${value})`);
query(sql`SELECT 1 FROM accounts WHERE id = CAST(${value} AS uuid)`);
