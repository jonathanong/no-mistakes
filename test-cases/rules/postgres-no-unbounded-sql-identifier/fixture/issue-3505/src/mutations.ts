import sql from "sql-template-strings";
import { read } from "db";

export async function textPropertyWrite(raw: string) {
  const q = sql`SELECT * FROM items`;
  q.text = raw;
  return read(`/* text write */ SELECT * FROM ${q.text}`, q.values);
}

export async function stringsPropertyWrite(table: string) {
  const q = sql`id FROM items`;
  q.strings = [table];
  return read(`/* property write */ SELECT ${q.text}`, q.values);
}
