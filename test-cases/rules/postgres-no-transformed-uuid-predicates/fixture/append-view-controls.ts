import sql from "sql-template-strings";
import { read } from "db";

export async function foreignValues() {
  const statement = sql`SELECT t.id FROM threads t`;
  const other = sql`SELECT 1`;
  return read(statement.text, other.values);
}

export async function extraArgument() {
  const statement = sql`SELECT t.id FROM threads t`;
  return read(statement.text, statement.values, {});
}

export async function optionalText() {
  const statement = sql`SELECT t.id FROM threads t`;
  return read(statement?.text, statement.values);
}

export async function mutatedBuilder() {
  const statement = sql`SELECT t.id FROM threads t`;
  statement.text = "SELECT 1";
  return read(statement.text, statement.values);
}

export async function unknownBuilder(statement: { text: string; values: unknown[] }) {
  return read(statement.text, statement.values);
}
