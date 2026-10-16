import { query } from "@example/db";

function withColumn(column: string) {
  return "SELECT id FROM topics WHERE ".append(column);
}

const sql = withColumn("post_id");

export function load() {
  return query(sql);
}
