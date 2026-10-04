import { query } from "@example/db";

const sql = "SELECT id FROM topics";
do sql.append(" WHERE id = 1");
while (flag);

export function load() {
  return query(sql);
}
