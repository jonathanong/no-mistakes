import sql from "sql-template-strings";
import { read } from "db";
import { imported } from "./exported";
import * as statements from "./namespace";

export async function direct() {
  return read(`SELECT * FROM ${process.env.DIRECT_TABLE}`);
}

export async function reassigned() {
  let q = sql`SELECT * FROM items`;
  q = { text: `SELECT * FROM ${process.env.REASSIGNED_TABLE}`, values: [] };
  return read(`/* reassigned */ ${q.text}`, q.values);
}

export async function missingValues() {
  const q = sql`SELECT * FROM ${process.env.MISSING_TABLE}`;
  return read(`/* missing values */ ${q.text}`);
}

export async function wrongValues() {
  const q = sql`SELECT * FROM ${process.env.WRONG_TABLE}`;
  const other = sql`SELECT 1 WHERE id = ${1}`;
  return read(`/* wrong values */ ${q.text}`, other.values);
}

function returned(id: string) {
  return sql`SELECT * FROM ${id}`;
}

export async function missingHelperValues(id: string) {
  const q = returned(id);
  return read(`/* helper values */ ${q.text}`);
}

export async function missingAppendValues(id: string) {
  const q = sql`SELECT * FROM items`;
  q.append(sql` WHERE id = ${id}`);
  return read(`/* append values */ ${q.text}`);
}

export async function missingImportedValues() {
  return read(`/* imported values */ ${imported.text}`);
}

function recursive(): any {
  return recursive();
}

export async function recursiveBuilder() {
  const q = recursive();
  return read(`/* recursive */ ${q.text}`, q.values);
}

export async function missingNamespaceValues() {
  return read(`/* namespace values */ ${statements.namespaceStatement.text}`);
}
