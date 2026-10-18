// An unmutated configured subpath is still a proven namespace.
const child = require('@example/db/child');
child.query('SELECT 11');
