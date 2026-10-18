import { beginTransaction } from '@example/db';
const transaction = beginTransaction();
// A deleted query member cannot retain the factory receiver proof.
delete transaction.query;
transaction.query({ resource: 'http' });
transaction('SELECT direct delete');
