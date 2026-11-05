import { query, sql } from "@example/db";
import { tables } from "./tables";

export async function consumers(condition: boolean) {
  await query(`SELECT * FROM accounts LIMIT ${condition ? "1" : "2"} OFFSET 0`);
  await query(`SELECT id FROM accounts WHERE lower(id::text) = 'x' AND 1 = ${condition ? "1" : "2"}`);
  await query(`SELECT id FROM accounts WHERE created_at > now() AND 1 = ${condition ? "1" : "2"}`);
  await query(`UPDATE accounts SET created_at = now() WHERE 1 = ${condition ? "1" : "2"}`);
  await query(`INSERT INTO accounts (id) VALUES ('00000000-0000-0000-0000-000000000001') RETURNING ${condition ? "1" : "2"}`);
  await query(`INSERT INTO accounts (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) WHERE 1 = ${condition ? "1" : "2"} ON CONFLICT (id) DO NOTHING`);
  await query(`SELECT id FROM accounts WHERE id = ANY($1::uuid[]) AND 1 = ${condition ? "1" : "2"} FOR UPDATE`);
  await query(`UPDATE orders SET details = '{"accountId": ${condition ? "1" : "2"}}'::jsonb`);
  await query(`TRUNCATE accounts; SELECT ${condition ? "1" : "2"}`);
  await query(`SELECT ${condition ? "1" : "2"} FROM accounts FOR UPDATE !!!`);
  await query(sql`SELECT 1::numeric(${sql.raw(`${condition ? "1" : "2"}`)}) FROM ${process.env.TABLE}`);
  await query(sql`SELECT 1::numeric(${sql.raw(`${condition ? "1" : "2"}`)}) FROM ${tables.accounts}`);
}
