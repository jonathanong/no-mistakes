import type { TransactionQuery } from './database.mts';
import type { TransactionQuery as OtherQuery } from './unselected.mts';
import { write } from './database.mts';
import { write as otherWrite } from './unselected.mts';
export async function typedVsUnselected(ids: string[], run: TransactionQuery) {
  const query = run ?? otherWrite;
  for (const id of ids) await query('SELECT ' + id);
}
export async function unselectedVsSelected(ids: string[], run: OtherQuery) {
  const query = run ?? write;
  for (const id of ids) await query('SELECT ' + id);
}
export async function unknownVsSelected(ids: string[], run) {
  const query = run ?? write;
  for (const id of ids) await query('SELECT ' + id);
}
export async function typedVsUnknown(ids: string[], run: TransactionQuery, fallback) {
  const query = run ?? fallback;
  for (const id of ids) await query('SELECT ' + id);
}
export async function unselectedVsUnknown(ids: string[], run: OtherQuery, fallback) {
  const query = run ?? fallback;
  for (const id of ids) await query('SELECT ' + id);
}
export async function unknownVsUnselected(ids: string[], run) {
  const query = run ?? otherWrite;
  for (const id of ids) await query('SELECT ' + id);
}
// A non-nullish literal cannot execute the fallback, even though it is a sink.
export async function unreachableFallback(ids: string[]) {
  const query = 'not callable' ?? write;
  for (const id of ids) await query('SELECT ' + id);
}
