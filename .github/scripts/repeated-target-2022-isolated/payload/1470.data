import { query } from "@example/db";

const sql = "SELECT id FROM topics";
switch (flag) {
  case 1:
    sql.append(" WHERE id = 1");
}

export function load() {
  return query(sql);
}
