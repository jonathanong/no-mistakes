import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

export function insert(ids: string[]) {
  const statement = sql`
    INSERT INTO items (id)
    SELECT input.id
    FROM unnest($1::uuid[]) AS input(id)
  `;
  statement.append(sql`
    ON CONFLICT (id) DO NOTHING
  `);
  return query(statement, [ids]);
}
