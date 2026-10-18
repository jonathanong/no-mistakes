import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare const flag: boolean;
declare const other: { query(input: object): unknown };
const box = { tx: flag ? tx : other };
box.tx.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct conditional container');
