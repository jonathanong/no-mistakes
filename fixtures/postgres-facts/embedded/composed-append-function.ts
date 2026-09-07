import { query } from "@data-stores/psql";

const sql = "INSERT INTO items (id) VALUES (1)";
function unused() {
  sql.append(" ON CONFLICT (id) DO NOTHING");
}

export function insertItem() {
  return query(sql);
}
