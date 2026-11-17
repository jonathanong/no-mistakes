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
  await read(q.text, q.values);
  return read(`/* stable let */ ${q.text}`, q.values);
}

export async function nestedOwner(id: string) {
  const group = { q: sql`SELECT * FROM items WHERE id = ${id}` };
  return read(`/* nested owner */ ${group.q.text}`, group.q.values);
}

declare function opaqueFactory(): { text: string; values: unknown[] };

export async function opaque() {
  const q = opaqueFactory();
  return read(`/* opaque */ ${q.text}`, q.values);
}

export async function structuralLocal(id: string) {
  const fragment = sql`SELECT * FROM items WHERE id = ${id}`;
  const q = { text: fragment.text, values: fragment.values };
  return read(`/* structural local */ ${q.text}`, q.values);
}

export async function parameterSource(q: { text: string; values: unknown[] }) {
  return read(`/* parameter source */ ${q.text}`, q.values);
}
