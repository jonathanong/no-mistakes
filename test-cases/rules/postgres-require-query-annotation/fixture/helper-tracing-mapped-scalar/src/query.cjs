const database = require('@app/db');
function unsafe(statement) {
  const escaped = opaque(arguments);
  const assigned = (arguments[0] = 'SELECT 1');
  database.query(statement); // finding:known-unsafe-possibility
}
function annotated(statement) {
  const escaped = opaque(arguments);
  const assigned = (arguments[0] = '/* annotation */ SELECT 1');
  database.query(statement); // unanalyzable:safe-possibility-does-not-prove-safety
}
function actualArray(statement) {
  const escaped = opaque(arguments);
  const assigned = (arguments[0] = ['SELECT 1']);
  database.query(statement); // unanalyzable:actual-array-is-not-sql
}
unsafe('/* original */ SELECT 1');
annotated('/* original */ SELECT 1');
actualArray('/* original */ SELECT 1');

function unsafeBeforeDisconnection(statement) {
  const escaped = opaque(arguments);
  arguments[0] = 'SELECT 1';
  delete arguments[0];
  database.query(statement); // finding:known-slot-before-disconnection
}
unsafeBeforeDisconnection('/* original */ SELECT 1');
