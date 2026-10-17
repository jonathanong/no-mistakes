import { primary, beginFactory, begin } from '@vendor/database';
import type { TransactionQuery } from '@vendor/database';
export function typed(transaction: TransactionQuery) {
  transaction('query');
}
const returned = beginFactory();
returned('query');
function queryHelper() { primary.read(); }
function direct() { begin(); queryHelper(); }
direct();
