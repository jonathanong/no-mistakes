import { beginTransaction } from '@example/db';
const transaction = beginTransaction();
// An unknown helper can replace `.query` through its receiver argument.
mutate(transaction);
transaction.query({ resource: 'http' });
transaction('SELECT direct escape');
