import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare function noop(): void;
// The last sequence value is the mutable receiver.
let alias = (noop(), tx);
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct sequence escape');
