import { query } from "@example/db";

const sql = "SELECT id FROM topics";
// Conditional static appends keep this recovered SELECT and classify Dynamic
// so conflict-ordering ignores it, rather than composing the branch as
// always-on SQL.
if (flag) sql.append(" WHERE id = 1");

export function load() {
  return query(sql);
}
