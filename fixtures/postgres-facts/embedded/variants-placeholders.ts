import { query, sql } from "@example/db";
const tail = flag
  ? sql`WHERE tenant_id = ${tenant}`
  : sql`WHERE owner_id = ${owner}`;
query(sql`SELECT ${value} FROM users ${tail} AND id > ${cursor} AND sql_placeholder_1 = 0`);
