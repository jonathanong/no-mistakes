import sql from "sql-template-strings";
import { read } from "@example/db";
import { visibleSql } from "./visibility-sql.mts";

// Two appends of one export inside one function. The view text is the helper's,
// so both uses are one finding on the helper.
export async function twice(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.owner_id"));
  query.append(sql` AND `);
  query.append(visibleSql("l.parent_id"));
  return read(query);
}

// One directive covers the whole statement: the two appends are one execution
// of the same helper token, not two findings that can be suppressed apart.
export async function oneOfTwo(id: string) {
  const query = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  query.append(visibleSql("l.owner_id"));
  query.append(sql` AND `);
  // no-mistakes-disable-next-line postgres-required-predicates
  query.append(visibleSql("l.parent_id"));
  return read(query);
}
