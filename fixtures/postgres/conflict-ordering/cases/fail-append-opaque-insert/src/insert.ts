import sql from "sql-template-strings";
import { query } from "@data-stores/psql";

declare const importedConflictClause: string;

export function insert(ids: string[]) {
  const statement = sql`
    INSERT INTO items (id)
    SELECT input.id
    FROM unnest($1::uuid[]) AS input(id)
  `;
  statement.append(importedConflictClause);
  return query(statement, [ids]);
}

export function insertFluent(ids: string[]) {
  return query(
    sql`
      INSERT INTO items (id)
      SELECT input.id
      FROM unnest($1::uuid[]) AS input(id)
    `.append(importedConflictClause),
    [ids],
  );
}
