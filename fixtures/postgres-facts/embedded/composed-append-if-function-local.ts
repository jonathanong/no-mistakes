import sql from "sql-template-strings";
import { read } from "@data-stores/psql";

export async function listTopics(flag: boolean) {
  const query = sql`SELECT id FROM topics WHERE published = true`;
  // Same as composed-append-if.ts: keep the recovered SELECT, do not compose
  // the branch-only ORDER BY as always-on SQL.
  if (flag) {
    query.append(sql` ORDER BY id`);
  }
  return read(query);
}
