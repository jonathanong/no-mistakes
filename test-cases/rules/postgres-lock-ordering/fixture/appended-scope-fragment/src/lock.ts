import { query as execute } from "@example/db";
import sql from "sql-template-strings";
import { scopeCondition } from "scope-conditions";
import { tableName } from "table-names";

declare const n: number;

function scopeSql(feedId: string) {
  return sql` AND source.feed_id = ${feedId}`;
}

export function claimResolved(feedId: string) {
  const statement = sql`WITH candidate AS (SELECT source.id FROM sources source WHERE source.published_at IS NULL`;
  statement.append(scopeSql(feedId));
  statement.append(
    sql` ORDER BY source.id LIMIT ${n} FOR UPDATE OF source SKIP LOCKED) UPDATE sources SET published_at = now() FROM candidate WHERE sources.id = candidate.id`,
  );
  return execute(statement);
}

// An unresolved fragment must stay opaque. It must not become a placeholder.
export function claimUnresolved() {
  const statement = sql`WITH candidate AS (SELECT source.id FROM sources source WHERE source.published_at IS NULL`;
  statement.append(scopeCondition);
  statement.append(
    sql` ORDER BY source.id LIMIT ${n} FOR UPDATE OF source SKIP LOCKED) UPDATE sources SET published_at = now() FROM candidate WHERE sources.id = candidate.id`,
  );
  return execute(statement);
}

// A fragment hole beside an identifier hole is still opaque.
export function claimWithTable() {
  const statement = sql`WITH candidate AS (SELECT source.id FROM `;
  statement.append(tableName);
  statement.append(sql` source WHERE source.published_at IS NULL`);
  statement.append(scopeCondition);
  statement.append(
    sql` ORDER BY source.id LIMIT ${n} FOR UPDATE OF source SKIP LOCKED) UPDATE sources SET published_at = now() FROM candidate WHERE sources.id = candidate.id`,
  );
  return execute(statement);
}
