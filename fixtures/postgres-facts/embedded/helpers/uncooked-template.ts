import sql from "sql-template-strings";
import { query } from "@example/db";
import { ownerVisible } from "./elsewhere";

function tail(clause: string): string {
  return `ORDER BY created_at ${clause}`;
}

// An invalid escape has no cooked value, so the argument must not be spliced.
export function badEscape() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible(`\8`));
  query(q);
  const local = sql`SELECT 1 `;
  local.append(tail(`\8`));
  query(local);
}
