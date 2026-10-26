import SQL from "sql-template-strings";
import { read, QueryExecutor } from "@example/db";
declare const flag: boolean;

function budgetIdentity(value: string) {
  return value;
}
export function conditionalBudget(enabled: unknown) {
  // Copies and nested summaries must consume the same per-root work budget.
  const statement = SQL`SELECT 1 OFFSET 9`;
  budgetIdentity("metadata");
  if (enabled) statement.append(SQL` /* more text */`);
  const selected = enabled ? statement : statement;
  const retained = (enabled && selected) || selected;
  return enabled ? retained : statement;
}
export function templateBudget(value = "a deliberately nonempty bind payload") {
  return SQL`SELECT ${value}, ${value} OFFSET 9`;
}
export function appendBudget() {
  return SQL`SELECT 1 OFFSET 9`.append(SQL` /* suffix */`);
}
read(conditionalBudget(flag));
read(templateBudget());
read(appendBudget());

export function executeBudget(query: QueryExecutor) {
  // A member-valued local is data; caller replay must still find the actual sink.
  const data = { field: "data" }.field;
  const statement = SQL`SELECT 1 OFFSET 9`;
  query(statement);
  return data;
}
executeBudget(read);
