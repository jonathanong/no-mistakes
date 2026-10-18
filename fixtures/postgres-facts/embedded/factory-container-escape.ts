import { beginTransaction } from '@example/db';
const transaction = beginTransaction();
const box = { transaction };
box.transaction.query = (_request: object) => undefined;
transaction.query({ resource: 'http' });
transaction('SELECT direct container escape');
