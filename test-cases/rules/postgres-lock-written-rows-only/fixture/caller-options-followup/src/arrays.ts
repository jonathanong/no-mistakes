import { write, beginTransaction } from '@example/db';
async function arrayLock(id: string, options: unknown) {
  await write('SELECT id FROM users WHERE id = $1 FOR UPDATE', [id], options);
}
async function unsupportedArray(options: unknown) {
  await write('SELECT id FROM users WHERE id = $1 FOR UPDATE', [getId()], options);
}
export async function arrayCalls(id: string) {
  await using tx = await beginTransaction();
  await arrayLock(id, { query: tx });
  await unsupportedArray({ query: tx });
}
