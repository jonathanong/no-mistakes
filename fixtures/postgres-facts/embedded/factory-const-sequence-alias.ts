import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare function noop(): void;
// The final sequence value escapes through an unproven const binding.
const alias = (noop(), tx);
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct const sequence');
