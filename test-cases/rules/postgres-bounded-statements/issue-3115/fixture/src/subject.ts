import { read, sql } from "@example/db";

export function bounded(id: string) {
  const subject = sql`SELECT id FROM accounts WHERE id = ${id}`;
  const query = sql`SELECT s.id FROM (`;
  query.append(subject).append(sql`) s`);
  return read(query);
}

export function unbounded(name: string) {
  const subject = sql`SELECT id FROM accounts WHERE name = ${name}`;
  const query = sql`SELECT s.id FROM (`;
  query.append(subject).append(sql`) s`);
  return read(query);
}
