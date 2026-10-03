import { query } from "fixture-db";
export function lock(ids: number[]) {
  return query(`SELECT * FROM "Catalog.Test"."Items" WHERE "Id" = ANY($1) ORDER BY "Id" FOR UPDATE`, [ids]);
}
