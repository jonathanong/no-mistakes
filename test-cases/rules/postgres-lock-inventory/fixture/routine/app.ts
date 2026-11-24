import { query } from "@example/db";

export async function install() {
  return query(`CREATE FUNCTION reserve_account() RETURNS bigint LANGUAGE sql AS $body$
    SELECT id FROM accounts FOR UPDATE;
  $body$;
    SELECT id FROM accounts FOR SHARE;`);
}
