import { query, sql } from "@example/db";
const shared = sql`SELECT id FROM accounts`;
// Both requirements per site must survive, while repeated alternatives deduplicate.
query(sql`${shared} ${flag ? sql`/* first */` : sql`/* second */`}`);
query(sql`${shared} ${otherFlag ? sql`WHERE active IS TRUE AND tenant_id = ${tenant}` : sql`WHERE deleted_at IS NULL AND region_id = ${region}`}`);
