import { beginTransaction } from '@example/db';
const tx = beginTransaction();
// A concise arrow leaks the receiver just like an explicit return.
const expose = () => tx;
expose().query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct arrow escape');
