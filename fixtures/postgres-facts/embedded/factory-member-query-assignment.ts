import { beginTransaction } from '@example/db';
const transaction = beginTransaction();
function saved() { return transaction.query('SELECT stale'); }
const alias = transaction;
alias.query = (_request: object) => undefined;
saved();
transaction.query({ resource: 'http' });
transaction('SELECT direct assignment');
