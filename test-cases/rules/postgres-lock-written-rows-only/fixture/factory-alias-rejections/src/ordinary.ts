function ordinary(_sql: string) {}
export async function unrelated() {
  const holder = { tx: ordinary, query: ordinary };
  const q = holder.tx;
  const { query } = holder;
  await q(`SELECT id FROM items FOR UPDATE`);
  await query(`SELECT id FROM items FOR UPDATE`);
  let execute = ordinary;
  execute = ordinary;
  await execute(`SELECT id FROM items FOR UPDATE`);
}
