import { query, sql } from "@example/db";

export function bounded(id: string) {
  const predicate = sql`WHERE x.account_id = target.id`;
  const statement = sql`SELECT target.id FROM accounts target
    JOIN LATERAL (SELECT x.id FROM orders x `;
  statement.append(predicate);
  statement.append(sql`) child ON true WHERE target.id = ${id}`);
  return query(statement);
}

export function unbounded() {
  const predicate = sql`WHERE x.account_id = target.id`;
  const statement = sql`SELECT target.id FROM accounts target
    JOIN LATERAL (SELECT x.id FROM orders x `;
  statement.append(predicate);
  statement.append(sql`) child ON true`);
  return query(statement);
}
