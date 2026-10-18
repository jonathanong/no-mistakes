import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare const flag: boolean;
declare const other: { query(input: object): unknown };
// This const cannot prove which mutable object its query property names.
const alias = flag ? tx : other;
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct const conditional');
