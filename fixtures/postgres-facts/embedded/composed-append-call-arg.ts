import { query } from "@data-stores/psql";

function idColumn() {
  return "post_id";
}

const sql = "SELECT id FROM topics WHERE ";
sql.append(idColumn());

export function load() {
  return query(sql);
}
