import { read, sql } from "@example/db";

export function bounded(topicId: string, limit: number) {
  const query = sql`SELECT id FROM accounts WHERE name = ${topicId} ORDER BY id`;
  query.append(sql` LIMIT ${limit}`);
  return read(query);
}

export function unbounded(topicId: string) {
  const query = sql`SELECT id FROM accounts WHERE name = ${topicId} ORDER BY id`;
  query.append(sql` OFFSET 1`);
  return read(query);
}
