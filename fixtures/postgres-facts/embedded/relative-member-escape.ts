import { openTransaction } from '../transaction';
const tx = openTransaction();
function saved() { return tx.query('SELECT stale relative'); }
const box = { tx };
box.tx.query = (_request: object) => undefined;
saved();
tx.query({ resource: 'http' });
tx('SELECT direct relative');
