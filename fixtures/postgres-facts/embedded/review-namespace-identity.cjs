// Requiring the exact same module shares an object; a subpath is distinct.
const first = require('@example/db');
const same = require('@example/db');
const alias = first;
const subpath = require('@example/db/tx');
first.query('SELECT id FROM namespace_before');
sendAway(alias);
first.query('SELECT id FROM namespace_first_after');
same.query('SELECT id FROM namespace_same_after');
subpath.query('SELECT id FROM namespace_subpath');
