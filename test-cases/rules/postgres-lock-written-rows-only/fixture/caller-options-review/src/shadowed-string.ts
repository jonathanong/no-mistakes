import { write, beginTransaction } from '@example/db';
const String = { raw(_parts: unknown) { return 'SELECT id FROM items FOR UPDATE'; } };
async function shadowed(opts?: { query?: unknown }) {
  await write(String.raw`SELECT 1`, opts);
}
async function run() {
  using tx = beginTransaction();
  await shadowed({ query: tx });
}
