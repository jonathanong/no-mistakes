import sql, { type SQLStatement } from "sql-template-strings";

export function appendOrdering(query: SQLStatement): SQLStatement {
  query.append(sql` ORDER BY t.account_id`);
  return query;
}
