import sql from "sql-template-strings";
import { query } from "@example/db";
import { openTransaction as outsideFactory, type TxExecutor as OutsideExecutor } from "../packages/other/src/transaction";
import { openTransaction as insideFactory, type TxExecutor as InsideExecutor } from "../packages/db/src/transaction";

export function rejectedFactory(gate: boolean) {
  const statement = gate ? sql`SELECT 1` : sql`SELECT 2`;
  const alias = statement;
  const unrelated = gate ? sql`SELECT 17` : sql`SELECT 18`;
  const execute = outsideFactory();
  execute(statement);
  query(statement); // findings: unanalyzable
  query(alias); // findings: unanalyzable
  query(unrelated);
}

export function rejectedType(execute: OutsideExecutor, gate: boolean) {
  const statement = gate ? sql`SELECT 3` : sql`SELECT 4`;
  const alias = statement;
  execute(alias);
  query(statement); // findings: unanalyzable
  query(alias); // findings: unanalyzable
}

export function acceptedFactory(gate: boolean) {
  const statement = gate ? sql`SELECT 5` : sql`SELECT 6`;
  const alias = statement;
  const execute = insideFactory();
  execute(statement); // The pending call itself still has finite SQL.
  query(statement); // findings: unanalyzable
  query(alias); // findings: unanalyzable
}

export function acceptedType(execute: InsideExecutor, gate: boolean) {
  const statement = gate ? sql`SELECT 7` : sql`SELECT 8`;
  const alias = statement;
  execute(alias);
  query(statement); // findings: unanalyzable
  query(alias); // findings: unanalyzable
}

export function copiedText(gate: boolean) {
  const statement = gate ? "SELECT 9" : "SELECT 10";
  const execute = insideFactory();
  execute(statement);
  query(statement);
}

export function confirmedConsumer(gate: boolean) {
  const statement = gate ? sql`SELECT 11` : sql`SELECT 12`;
  query(statement);
  query(statement);
}

export function capturedConsumer(gate: boolean) {
  const statement = gate ? sql`SELECT 13` : sql`SELECT 14`;
  function nested(execute: OutsideExecutor) {
    execute(statement);
    query(statement); // findings: unanalyzable
  }
  // Inspecting the nested function does not mutate the enclosing execution.
  query(statement);
  return nested;
}

export function suppressedReuse(gate: boolean) {
  const statement = gate ? sql`SELECT 15` : sql`SELECT 16`;
  const execute = outsideFactory();
  execute(statement);
  // no-mistakes-disable-next-line postgres-no-offset
  query(statement); // suppressed: unanalyzable
  query(statement); // findings: unanalyzable
}

export function acceptedFactoryArguments(gate: boolean) {
  const statement = sql`SELECT id FROM factory_users`;
  if (gate) statement.append(" WHERE id = 1");
  const execute = insideFactory();
  execute(statement, statement.append(" OFFSET 1")); // findings: offset
  query(gate ? statement : sql`SELECT 19`); // findings: unanalyzable
}

export function acceptedTypeArguments(execute: InsideExecutor, gate: boolean) {
  const statement = sql`SELECT id FROM typed_users`;
  if (gate) statement.append(" WHERE id = 2");
  execute(statement, statement.append(" OFFSET 2")); // findings: offset
  query(gate ? statement : sql`SELECT 20`); // findings: unanalyzable
}

export function rejectedFactoryArguments(gate: boolean) {
  const statement = sql`SELECT id FROM outside_factory_users`;
  if (gate) statement.append(" WHERE id = 3");
  const execute = outsideFactory();
  execute(statement, statement.append(" LIMIT 1"));
  query(gate ? statement : sql`SELECT 21`); // findings: unanalyzable
}

export function rejectedTypeArguments(execute: OutsideExecutor, gate: boolean) {
  const statement = sql`SELECT id FROM outside_typed_users`;
  if (gate) statement.append(" WHERE id = 4");
  execute(statement, statement.append(" LIMIT 1"));
  query(gate ? statement : sql`SELECT 22`); // findings: unanalyzable
}
