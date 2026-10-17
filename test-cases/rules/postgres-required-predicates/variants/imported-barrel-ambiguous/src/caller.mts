import sql from "sql-template-strings";
import { read } from "@example/db";
import { visibleSql } from "./index.mts";

// Two `export *` sources declare different `visibleSql` functions. ES modules
// treat such a name as ambiguous, so neither declaration may be assumed.
export async function listLinks(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.owner_id"));
  return read(query);
}
