import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

function buildInsert(ids: string[]) {
  const statement = sql`
    WITH input AS (
      SELECT id FROM unnest($1::uuid[]) AS input(id)
    )
  `;
  statement.append(sql`
    INSERT INTO items (id)
    SELECT input.id
    FROM input
    ORDER BY input.id
    ON CONFLICT (id) DO NOTHING
  `);
  return statement;
}

export function insert(ids: string[]) {
  return query(buildInsert(ids), [ids]);
}
