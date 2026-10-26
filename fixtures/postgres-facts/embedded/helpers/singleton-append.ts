import sql from "sql-template-strings";
import { query } from "@example/db";
import { ownerVisible } from "./elsewhere";

// A deterministic append mirrors complete SQL in raw text and one variant.
export function singleton(id: string) {
  const q = sql`SELECT ${id} WHERE `;
  q.append(ownerVisible("x"));
  q.append(sql` AND suffix = ${id}`);
  query(q);
}
