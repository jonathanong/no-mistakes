import { query } from "@data-stores/psql";

const sql = "INSERT INTO items (id) VALUES (1)";
// Nested unused function mutating an outer SQL binding must fail closed:
// walking the declaration is not executing it, so applying this ON CONFLICT
// would analyze SQL the runtime INSERT never runs.
function unused() {
  sql.append(" ON CONFLICT (id) DO NOTHING");
}

export function insertItem() {
  return query(sql);
}
