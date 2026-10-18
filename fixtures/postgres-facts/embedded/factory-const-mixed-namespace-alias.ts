import * as ns from '@example/db';
import { beginTransaction } from '@example/db';
const tx = beginTransaction();
declare const flag: boolean;
// Either executor category can be mutated through this one alias.
const alias = flag ? ns : tx;
alias.query = (_request: object) => undefined;
tx.query({ resource: 'http' });
ns.query({ resource: 'http' });
tx('SELECT direct const mixed namespace');
