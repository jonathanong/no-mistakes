import sql from "sql-template-strings";
import { query } from "@example/db";
import { openTransaction, type TxExecutor } from "./variants-executor-support";

export function factoryConsumer(gate: boolean) {
  const statement = gate ? sql`SELECT 1` : sql`SELECT 2`;
  const alias = statement;
  const unrelated = gate ? sql`SELECT 11` : sql`SELECT 12`;
  const execute = openTransaction();
  execute(statement); // relative finite
  query(statement); // opaque
  query(alias); // opaque
  query(unrelated); // confirmed finite
}

export function typedConsumer(execute: TxExecutor, gate: boolean) {
  const statement = gate ? sql`SELECT 3` : sql`SELECT 4`;
  const alias = statement;
  execute(alias); // relative finite
  query(statement); // opaque
  query(alias); // opaque
}

export function copiedText(gate: boolean) {
  const statement = gate ? "SELECT 5" : "SELECT 6";
  const execute = openTransaction();
  execute(statement); // relative finite
  query(statement); // confirmed finite
}

export function confirmedConsumer(gate: boolean) {
  const statement = gate ? sql`SELECT 7` : sql`SELECT 8`;
  query(statement); // confirmed finite
  query(statement); // confirmed finite
}

export function capturedConsumer(gate: boolean) {
  const statement = gate ? sql`SELECT 9` : sql`SELECT 10`;
  function nested(execute: TxExecutor) {
    execute(statement); // relative opaque
    query(statement); // opaque
  }
  query(statement); // confirmed finite
  return nested;
}
