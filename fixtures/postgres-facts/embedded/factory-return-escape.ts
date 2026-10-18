import { beginTransaction } from '@example/db';
const tx = beginTransaction();
function expose() { return tx; }
expose().query = (_request: object) => undefined;
tx.query({ resource: 'http' });
tx('SELECT direct return escape');
