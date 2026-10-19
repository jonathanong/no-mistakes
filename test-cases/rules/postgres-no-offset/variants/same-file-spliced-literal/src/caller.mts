import sql from "sql-template-strings";
import { read } from "@example/db";

function tail(clause: string): string {
  return `ORDER BY created_at ${clause}`;
}

// OFFSET is written only in the quoted string. The finding belongs on that
// line, not on `${clause}` above.
export async function page(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  query.append(tail("LIMIT 20 OFFSET 40"));
  return read(query);
}
