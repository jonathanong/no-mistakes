import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare const flag: boolean;
declare const other: { query(input: object): unknown };
declare function mutate(value: unknown): void;
mutate(flag ? tx : other);
tx.query({ resource: 'http' });
tx('SELECT direct conditional argument');
