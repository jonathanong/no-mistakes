import { query, sql } from "@example/db";
const shared = sql`SELECT id FROM accounts`;
query(sql`${shared} ${flag ? sql`WHERE active IS TRUE AND tenant_id = ${tenant}` : sql`WHERE deleted_at IS NULL AND region_id = ${region}`}`);
