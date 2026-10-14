import { sql } from '@example/db';
import * as db from '@example/db';
export const query = sql`SELECT 1`;

export function receiverGetter(object: unknown) {
  (object.mutateBuiltins, db).query(query);
}
export function computedReceiverGetter(object: unknown) {
  (object.mutateBuiltins, db)['query'](query);
}
export function templateReceiverGetter(object: unknown) {
  (object.mutateBuiltins, db)[`query`](query);
}
export function chainedReceiverGetter(object: unknown) {
  object.database.query(query);
}
export function optionalReceiverGetter(object: unknown) {
  (object.mutateBuiltins, db)?.query(query);
}
export function pureReceiver() {
  (0, db).query(query);
}
function pure() { return 0; }
export function knownReceiver() {
  (pure(), db).query(query);
}
export function namespaceReceiver() {
  db.query(query);
}
export function ordinaryOptionalReceiver() {
  db?.query('/* callees/optional */ SELECT 1');
}
export function optionalCallGetter(object: unknown) {
  (object.mutateBuiltins, db).query?.(query);
}
export function combinedOptionalGetter(object: unknown) {
  (object.mutateBuiltins, db)?.query?.(query);
}
export function ordinaryOptionalCall() {
  db.query?.('/* callees/optional-call */ SELECT 1');
}
export function ordinaryCombinedOptional() {
  db?.query?.('/* callees/combined */ SELECT 1');
}

// A member-only chain retains the existing opaque projection, not a call event.
export function optionalMember() {
  return db?.query;
}
