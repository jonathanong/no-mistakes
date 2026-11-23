import { query, sql } from "@example.invalid/db";

declare const replace: boolean;

await query(replace
  ? sql`CREATE EXTENSION IF NOT EXISTS widgets VERSION '1.4'`
  : sql`CREATE EXTENSION IF NOT EXISTS widgets`);
await query(replace
  ? sql`DROP VIEW widget_summary CASCADE`
  : sql`DROP VIEW widget_summary RESTRICT`);
