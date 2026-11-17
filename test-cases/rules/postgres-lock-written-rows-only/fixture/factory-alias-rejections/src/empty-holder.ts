import { query as pool } from "@example/db";
function ordinary(_sql: string) {}
export async function unrelated() {
  const empty = {};
  const holder = { ...empty, query: ordinary };
  const { query: callback } = holder;
  await callback(`SELECT id FROM items FOR UPDATE`);
}
