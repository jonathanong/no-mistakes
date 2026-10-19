import sql from "sql-template-strings";
import { read } from "@example/db";
import { tail } from "./paging.mts";

// OFFSET is only in the no-substitution template below, not on `${clause}`.
export async function page(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  query.append(tail(`LIMIT 20 OFFSET 40`));
  return read(query);
}
