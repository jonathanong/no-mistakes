import type { TransactionQuery } from './database.mts';
import type { TransactionQuery as OtherQuery } from './unselected.mts';
import { write as otherWrite } from './unselected.mts';
async function selected(operation: (query: TransactionQuery) => Promise<void>) { await operation(otherWrite); }
async function unselected(operation: (query: OtherQuery) => Promise<void>) { await operation(otherWrite); }
// Both authored contexts apply to one callback; neither can borrow the other type.
export async function mixed(ids: string[]) {
  const shared = async query => { for (const id of ids) await query(id); };
  await selected(shared);
  await unselected(shared);
}
async function changed(operation: (query: TransactionQuery) => Promise<void>, unknown) {
  operation = unknown;
  await operation(otherWrite);
}
export async function reassigned(ids: string[], unknown) {
  await changed(async query => { for (const id of ids) await query(id); }, unknown);
}
type CallbackOptions = { operation: (query: TransactionQuery) => Promise<void> };
async function indexed(operation: CallbackOptions['operation']) { await operation(otherWrite); }
export async function unsupportedContext(ids: string[]) {
  await indexed(async query => { for (const id of ids) await query(id); });
}
