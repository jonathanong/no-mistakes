import { query, sql } from "@example/db";

export function unrecoverable(makePrefix: () => unknown) {
  const predicate = sql`WHERE x.account_id = target.id`;
  const statement = sql``;
  statement.append(makePrefix());
  statement.append(predicate);
  return query(statement);
}
