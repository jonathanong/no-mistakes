import sql from "sql-template-strings";
import { read } from "@example/db";
import { maybe } from "./maybe.mts";

function start(id: string) {
  return sql`SELECT r.id FROM roots r WHERE r.owner = ${id}`;
}

// Four holes with two alternatives each are sixteen statements: the cap.
export async function sixteen(id: string, flag: boolean) {
  const query = start(id);
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  return read(query);
}

// A fifth hole is thirty-two statements, which stays unanalyzable.
export async function thirtyTwo(id: string, flag: boolean) {
  const query = start(id);
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  query.append(sql` AND `).append(maybe(flag));
  return read(query);
}

// The caller's own alternatives multiply with the hole's.
export async function callerAlternatives(id: string, flag: boolean, other: boolean) {
  const query = start(id);
  if (other) {
    query.append(sql` AND r.kind = 'x'`);
  }
  query.append(sql` AND `).append(maybe(flag));
  return read(query);
}
