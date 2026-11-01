import { query } from "@example/db";

export async function viaBind(table: string, ids: string[]) {
  await query(`SELECT p.id FROM ${table} p JOIN unnest($1::uuid[]) AS u(id) ON u.id = p.id`, [ids]);
}

export async function rawHoles(table: string, ids: string[]) {
  await query(`SELECT p.id FROM ${table} p JOIN unnest(${ids}::uuid[]) AS u(id) ON u.id = p.id`);
}

export async function qualified(table: string) {
  await query(`SELECT id FROM public.${table}`);
}
