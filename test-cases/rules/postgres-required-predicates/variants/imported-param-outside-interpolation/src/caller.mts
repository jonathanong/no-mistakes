import sql from "sql-template-strings";
import { read } from "@example/db";
import { concatenated, transformed, reused } from "./visibility-sql.mts";

// Every argument is a literal, but each helper uses its parameter somewhere
// other than a `${param}` interpolation of the one returned template.
export async function viaConcatenation(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(concatenated("l.parent_id"));
  return read(query);
}

export async function viaTransformation(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(transformed("l.parent_id"));
  return read(query);
}

export async function viaStatementBeforeReturn(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(reused("l.parent_id"));
  return read(query);
}
