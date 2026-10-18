const base = require('@example/db');
const child = require('@example/db/child');
base.query = (_params) => undefined;
// A subpath can re-export the same mutable object, so its proof is revoked.
child.query('SELECT 10');
