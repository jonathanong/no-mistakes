import { query as pool } from "@example/db";
function ordinary(_sql: string) {}
export async function unrelated() {
  const holder = { query: pool, ordinary };
  const key = "ordinary";
  const { [key]: dynamic, ordinary: fallback = ordinary } = holder;
  await dynamic(`SELECT id FROM items FOR UPDATE`);
  await fallback(`SELECT id FROM items FOR UPDATE`);
  let mutable = { query: pool, ordinary };
  const callback = mutable.ordinary;
  await callback(`SELECT id FROM items FOR UPDATE`);
}
