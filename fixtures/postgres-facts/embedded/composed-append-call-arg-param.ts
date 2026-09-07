import { query } from "@data-stores/psql";

function withColumn(column: string) {
  return "SELECT id FROM topics WHERE ".append(column);
}

const sql = withColumn("post_id");

export function load() {
  return query(sql);
}
