import sql from "sql-template-strings";
import { read } from "@example/db";
import { orderBy } from "./paging.mts";

// Without the helper's text this query is unbounded; with it, nothing bounds it either.
export async function everyAccount() {
  const query = sql`SELECT id FROM accounts `;
  query.append(orderBy("email"));
  return read(query);
}
