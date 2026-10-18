import { beginTransaction } from '@example/db';
const transaction = beginTransaction();
let alias = transaction;
alias.query = (_request: object) => undefined;
transaction.query({ resource: 'http' });
transaction('SELECT direct mutable alias');
