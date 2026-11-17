import sql from "sql-template-strings";
import { read } from "db";

export async function direct(id: string) {
  const q = sql`SELECT * FROM items`.append(sql` WHERE id = ${id}`);
  await read(q.text, q.values);
  const sourceQuery = sql`SELECT * FROM items`.append(sql` WHERE id = ${id}`);
  return read(sourceQuery.text, sourceQuery.values);
}

export async function stableLet(id: string) {
  let q = sql`SELECT * FROM items WHERE id = ${id}`;
  return read(q.text, q.values);
}
