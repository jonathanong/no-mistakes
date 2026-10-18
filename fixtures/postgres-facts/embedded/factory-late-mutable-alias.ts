import { beginTransaction } from '@example/db';
const tx = beginTransaction();
let alias;
alias = tx;
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct late mutable');
