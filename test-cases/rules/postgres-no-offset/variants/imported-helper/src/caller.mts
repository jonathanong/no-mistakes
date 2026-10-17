import sql from "sql-template-strings";
import { read } from "@example/db";
import { pageClause } from "./paging.mts";

// The OFFSET lives in an imported helper's template; the finding belongs on its line.
export async function page(id: string) {
  const query = sql`SELECT id FROM accounts WHERE owner_id = ${id} `;
  query.append(pageClause("created_at", "20"));
  return read(query);
}
