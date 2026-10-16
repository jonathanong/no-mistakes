import type { TransactionQuery } from './database.mts';
import type { TransactionQuery as OtherQuery } from './unselected.mts';
import { write as otherWrite } from './unselected.mts';
async function selected(operation: (query: TransactionQuery) => Promise<void>) { await operation(otherWrite); }
async function same(operation: (query: TransactionQuery) => Promise<void>) { await operation(otherWrite); }
async function other(operation: (query: OtherQuery) => Promise<void>) { await operation(otherWrite); }
async function relayMixed(operation) { await selected(operation); await other(operation); }
async function relayAgreed(operation) { await selected(operation); await same(operation); }
// Forwarded inline callbacks retain every authored calling context.
export async function mixed(ids: string[]) {
  await relayMixed(async query => {
    for (const id of ids) await query(id);
  });
}
export async function agreed(ids: string[]) {
  await relayAgreed(async query => {
    for (const id of ids) await query(id);
  });
}
