import sql from "sql-template-strings";
import { read } from "@example/db";
import { lateralRows } from "./lateral.mts";

// The `*` projection lives in an imported helper, spliced between two fragments.
export async function accounts(id: string) {
  const query = sql`SELECT a.id FROM accounts a `;
  query.append(lateralRows("accounts"));
  query.append(sql` WHERE a.id = ${id}`);
  return read(query);
}
