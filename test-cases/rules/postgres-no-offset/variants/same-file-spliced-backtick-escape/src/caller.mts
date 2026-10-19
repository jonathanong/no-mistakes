import sql from "sql-template-strings";
import { read } from "@example/db";

function tail(clause: string): string {
  return `ORDER BY created_at ${clause}`;
}

// OFFSET is written only in the no-substitution template. The finding belongs
// on that line, not on `${clause}` above.
export async function page(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  // The escape must be read cooked, so the OFFSET is visible; do not change it to a plain space.
  query.append(tail(`LIMIT 20 \x20OFFSET 40`));
  return read(query);
}
