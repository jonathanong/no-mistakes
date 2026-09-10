import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

export function insert(ids: string[], ordered: boolean) {
  const statement = sql`
    INSERT INTO items (id)
    SELECT input.id
    FROM unnest($1::uuid[]) AS input(id)
  `;
  // Branch-only ORDER BY / ON CONFLICT must not compose as always-on, or
  // conflict-ordering would check the ordered path while the unordered path
  // can still run.
  if (ordered) {
    statement.append(sql`
      ORDER BY input.id
      ON CONFLICT (id) DO NOTHING
    `);
  }
  return query(statement, [ids]);
}
