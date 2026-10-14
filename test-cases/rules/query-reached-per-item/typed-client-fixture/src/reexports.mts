import type { Transaction } from './types.mjs';

export async function reexported(ids: string[], execute: Transaction) {
  for (const id of ids) await execute('SELECT $1', [id]);
}
// A configured annotation cannot make a proven non-callable default into a sink.
export function objectDefault(ids: string[], execute: Transaction = {} as any) {
  for (const id of ids) execute(id);
}
