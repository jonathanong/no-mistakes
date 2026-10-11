import { query, sql } from "@example/db";
query(sql`SELECT 1 ${flag0 ? sql` /* yes */` : sql` /* no */`}${flag1 ? sql` /* yes */` : sql` /* no */`}${flag2 ? sql` /* yes */` : sql` /* no */`}${flag3 ? sql` /* yes */` : sql` /* no */`}${flag4 ? sql` /* yes */` : sql` /* no */`}`);
