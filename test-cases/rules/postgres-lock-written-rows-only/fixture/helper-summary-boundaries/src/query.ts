import { beginTransaction, type TransactionQuery } from '@example/db';
export const nonFunction = 1;
export function* generator(query: TransactionQuery) { yield query('SELECT 1'); }
export default function(query: TransactionQuery) { return query('SELECT 1'); }
async function leaf(query: TransactionQuery, key: string) {
  return query('UPDATE claims SET value = $2 WHERE key = $1', [key, 'literal']);
}
async function objectLeaf(tx: unknown, key: string) {
  return tx.query('UPDATE claims SET value = 1 WHERE key = $1', [key]);
}
async function forwardedObject(tx: unknown, key: string) { return leaf(tx.query, key); }
async function forwardedReturn(query: TransactionQuery, key: string) { return leaf(query, key); }
async function unknownLeaf(query: TransactionQuery, key: string) {
  return query('UPDATE claims SET value = $2 WHERE key = $1', [key, external]);
}
async function forwardedUnknown(query: TransactionQuery, key: string) { return unknownLeaf(query, key); }
async function badLiteral(query: TransactionQuery, key: string) { return leaf(42, key); }
async function badOther(tx: unknown, key: string) { return leaf(tx.other, key); }
async function badOptional(tx: unknown, key: string) { return leaf(tx?.query, key); }
async function badExternal(tx: unknown, key: string) { return leaf(external.query, key); }
async function badObject(tx: unknown, key: string) { return objectLeaf(tx.query, key); }
async function badReceiver(query: TransactionQuery, key: string) { return query['other']('UPDATE claims SET value=1'); }
async function directLimit(query: TransactionQuery) {
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
  await query('SELECT 1');
}
async function double0(query: TransactionQuery) { return query('SELECT 1'); }
async function double1(query: TransactionQuery) { await double0(query); return double0(query); }
async function double2(query: TransactionQuery) { await double1(query); return double1(query); }
async function double3(query: TransactionQuery) { await double2(query); return double2(query); }
async function double4(query: TransactionQuery) { await double3(query); return double3(query); }
async function double5(query: TransactionQuery) { await double4(query); return double4(query); }
async function double6(query: TransactionQuery) { await double5(query); return double5(query); }
async function double7(query: TransactionQuery) { await double6(query); return double6(query); }
async function double8(query: TransactionQuery) { await double7(query); return double7(query); }
async function double9(query: TransactionQuery) { await double8(query); return double8(query); }
export async function caller_forwardedObject(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-clean
  await forwardedObject(tx, key);
}
export async function caller_forwardedReturn(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-clean
  await forwardedReturn(tx.query, key);
}
export async function caller_forwardedUnknown(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-clean
  await forwardedUnknown(tx.query, key);
}
export async function caller_badLiteral(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await badLiteral(tx.query, key);
}
export async function caller_badOther(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await badOther(tx, key);
}
export async function caller_badOptional(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await badOptional(tx, key);
}
export async function caller_badExternal(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await badExternal(tx, key);
}
export async function caller_badObject(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await badObject(tx, key);
}
export async function caller_badReceiver(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await badReceiver(tx.query, key);
}
export async function caller_directLimit(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await directLimit(tx.query, key);
}
export async function caller_double9(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await double9(tx.query, key);
}
async function safeBindings(tx: unknown, key: string) {
  await tx.query('UPDATE claims SET value = $2 WHERE key = $1', [key, -1, 'text', true, null, `opaque`]);
}
async function bindingCommit(tx: unknown, key: string) {
  await tx.query('SELECT 1', [tx.commit()]);
}
async function bindingAssignment(tx: unknown, key: string) {
  await tx.query('SELECT 1', [tx.query = replacement]);
}
async function bindingGetter(tx: unknown, key: string) {
  await tx.query('SELECT 1', [external.value]);
}
export async function caller_safeBindings(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-clean
  await safeBindings(tx, key);
}
export async function caller_bindingCommit(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await bindingCommit(tx, key);
  await tx.query('DELETE FROM claims WHERE key = $1', [key]);
}
export async function caller_bindingAssignment(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await bindingAssignment(tx, key);
  await tx.query('DELETE FROM claims WHERE key = $1', [key]);
}
export async function caller_bindingGetter(key: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT 1 FROM claims WHERE key = $1 FOR UPDATE', [key]); // expect-unresolved
  await bindingGetter(tx, key);
  await tx.query('DELETE FROM claims WHERE key = $1', [key]);
}
