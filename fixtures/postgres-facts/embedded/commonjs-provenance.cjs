const database = require('@example/db');
database.query('SELECT 1');
const alias = database;
alias.query('SELECT 2');
const { query: execute } = database;
execute('SELECT 3');
require('@example/db').query('SELECT 4');
// Importing a module namespace does not make the namespace callable.
database('SELECT 5');
alias('SELECT 6');
require('@example/db')('SELECT 7');

require('@example/http').query({ resource: 'http' });
function shadowRequire(require) {
  // This require is a parameter, so the configured module string proves nothing.
  const local = require('@example/db');
  local.query({ resource: 'http' });
}
function shadowDatabase(database) {
  database.query({ resource: 'http' });
}
function rebound() {
  let local = require('@example/db');
  local = require('@example/http');
  local.query({ resource: 'http' });
}
function replacedQuery() {
  const db = require('@example/db');
  db.query = (_params) => undefined;
  db.query({ resource: 'http' });
}
function aliasWrite() {
  const db = require('@example/db');
  const alias = db;
  alias.query = (_params) => undefined;
  db.query({ resource: 'http' });
}
function removedQuery() {
  const db = require('@example/db');
  delete db['query'];
  db.query({ resource: 'http' });
}
function conditionalNamespace(flag) {
  const db = flag ? require('@example/db') : require('@example/db');
  db('SELECT 8');
  const mixed = flag ? db : execute;
  mixed('SELECT 9');
}
function cachedModuleWrite() {
  const a = require('@example/db');
  const b = require('@example/db');
  a.query = (_params) => undefined;
  b.query({ resource: 'http' });
}
function escapedNamespace() {
  const db = require('@example/db');
  mutate(db);
  db.query({ resource: 'http' });
  const { query: afterMutation } = require('@example/db');
  afterMutation({ resource: 'http' });
}
