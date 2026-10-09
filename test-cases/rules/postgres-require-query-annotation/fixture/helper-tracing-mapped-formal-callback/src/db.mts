import type { SQLStatement } from "sql-template-strings";

export async function write(statement: SQLStatement): Promise<unknown> {
  return statement;
}
