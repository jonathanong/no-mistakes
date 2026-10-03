import { query } from "fixture-db";
export function insert(ids: number[], emails: string[]) {
  return query(`INSERT INTO "Catalog.Test"."Items" ("Id", email, active)
    SELECT input.id, input.email, true FROM unnest($1::bigint[], $2::text[]) input(id, email)
    ORDER BY input.email
    ON CONFLICT ON CONSTRAINT "Email.Unique" DO NOTHING`, [ids, emails]);
}
