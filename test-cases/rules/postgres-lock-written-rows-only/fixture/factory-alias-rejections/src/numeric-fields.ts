import { query as pool } from "@example/db";
function ordinary(_sql: string) {}
export async function unrelated() {
  const holder = { query: pool, 1: ordinary };
  const { 1: callback } = holder;
  await callback(`SELECT id FROM items FOR UPDATE`);
}
