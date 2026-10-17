import sql from "sql-template-strings";
import { read } from "@example/db";

// Reads the guarded view by a non-key column through the builder's `.text`.
export async function straight(tier: string) {
  const builder = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = ${tier}`;
  builder.append(sql` AND p.id IS NOT NULL`);
  return read(`/* straight */ WITH picked AS (${builder.text}) SELECT * FROM picked`, builder.values);
}

// Both alternatives of a conditional builder are checked.
export async function branching(tier: string, strict: boolean) {
  const builder = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = ${tier}`;
  if (strict) {
    builder.append(sql` AND p.id IS NOT NULL`);
  }
  return read(`/* branching */ WITH picked AS (${builder.text}) SELECT * FROM picked`, builder.values);
}

// An unknown builder: its `.text` is not recoverable SQL.
export async function unknownBuilder(builder: { text: string; values: unknown[] }) {
  return read(`/* unknown */ WITH picked AS (${builder.text}) SELECT * FROM picked`, builder.values);
}

// A second interpolation makes the template unrecoverable.
export async function extraInterpolation(tier: string, table: string) {
  const builder = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = ${tier}`;
  return read(`WITH picked AS (${builder.text}) SELECT * FROM ${table}`, builder.values);
}

// The bind array must be the same builder's `.values`.
export async function foreignValues(tier: string) {
  const builder = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = ${tier}`;
  const other = sql`SELECT 1`;
  return read(`WITH picked AS (${builder.text}) SELECT * FROM picked`, other.values);
}

// A plain object has a `.text` that is not SQL we can see.
export async function notABuilder() {
  const builder = { text: "SELECT p.id FROM view_active_plans p", values: [] };
  return read(`WITH picked AS (${builder.text}) SELECT * FROM picked`, builder.values);
}

// A builder grown by an unrecoverable append is not fully recovered.
export async function partialBuilder(tier: string, extra: string) {
  const builder = sql`SELECT p.id FROM view_active_plans p WHERE p.tier = ${tier}`;
  builder.append(extra);
  return read(`WITH picked AS (${builder.text}) SELECT * FROM picked`, builder.values);
}
