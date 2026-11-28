import { query } from "@example/db";
import sql, { SQLStatement } from "sql-template-strings";

const RETURNING_ALL = "RETURNING *";

export function updateEntriesConcat(id: string) {
  return query(
    "UPDATE entries SET status = $1 WHERE id = $2 " + RETURNING_ALL,
    ["done", id],
  );
}

export function deleteRecordsTemplate(id: string) {
  return query(`DELETE FROM records WHERE id = $1 ${RETURNING_ALL}`, [id]);
}

export function updateEntriesDynamicTemplate(dynamicSet: string, id: string) {
  return query(
    `UPDATE entries SET ${dynamicSet} WHERE id = $1 ${RETURNING_ALL}`,
    [id],
  );
}

export function updateRecordsBuilderAppend(
  status: string,
  id: string,
  filter: SQLStatement | undefined,
) {
  const q = sql`UPDATE records SET status = ${status} WHERE id = ${id}`;
  if (filter) q.append(filter);
  q.append(` ${RETURNING_ALL}`);
  return query(q);
}
