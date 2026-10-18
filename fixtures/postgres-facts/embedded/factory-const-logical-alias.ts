import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare const other: { query(input: object): unknown };
// The logical result may expose the configured receiver.
const alias = tx || other;
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct const logical');
