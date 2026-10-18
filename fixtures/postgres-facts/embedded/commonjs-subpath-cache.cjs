const base = require('@example/db');
const child = require('@example/db/child');
base.query = (_params) => undefined;
// Exact configured subpaths have independent proof identities.
child.query('SELECT 10');
