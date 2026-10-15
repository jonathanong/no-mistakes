import { write } from './database.mts';
import type { TransactionQuery } from './database.mts';
declare const unknown;
const leaf = unknown ?? write;
async function receiver(ids: string[]) {
  const box = { query: leaf, other: 0 };
  box.other = 1;
  const alias = box;
  for (const id of ids) {
    // A copied member keeps its own repeated acquisition and live sibling proof.
    const copy = alias.query;
    const left = copy ?? leaf;
    const right = leaf ?? copy;
    const query = left ?? right;
    await query(id);
  }
}
function acquire(input: TransactionQuery, unknown) {
  const left = input ?? unknown;
  const right = unknown ?? input;
  return left ?? right;
}
export async function entry(ids: string[], typed: TransactionQuery, unknown) {
  await receiver(ids);
  const query = unknown ?? acquire(typed, unknown);
  for (const id of ids) await query(id);
}
// The positive candidate must come from the copied member, not another leaf.
export async function copiedMember(ids: string[], unknown) {
  const box = { query: write };
  const alias = box;
  for (const id of ids) {
    const copy = alias.query;
    const query = copy ?? unknown;
    await query(id);
  }
}
