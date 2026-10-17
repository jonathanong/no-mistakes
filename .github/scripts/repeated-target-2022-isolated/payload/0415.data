import sql from "sql-template-strings";
import { query } from "@example/db";
import { openTransaction, type TxExecutor } from "./variants-executor-support";

export function factoryArguments(gate: boolean) {
  const statement = sql`SELECT id FROM factory_users`;
  if (gate) statement.append(sql` WHERE id = 1`);
  const execute = openTransaction();
  execute(statement, statement.append(sql` OFFSET 1`));
  query(gate ? statement : sql`SELECT 1`);
}

export function typedArguments(execute: TxExecutor, gate: boolean) {
  const statement = sql`SELECT id FROM typed_users`;
  if (gate) statement.append(sql` WHERE id = 2`);
  execute(statement, statement.append(sql` OFFSET 2`));
  query(gate ? statement : sql`SELECT 2`);
}
