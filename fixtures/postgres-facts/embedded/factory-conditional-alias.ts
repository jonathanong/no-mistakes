import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare const flag: boolean;
declare const other: { query(input: object): unknown };
let alias = flag ? tx : other;
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct conditional alias');
