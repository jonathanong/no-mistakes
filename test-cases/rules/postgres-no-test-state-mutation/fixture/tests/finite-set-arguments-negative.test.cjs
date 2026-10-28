const database = require('@example/db');

function mutate(args) { args[0] = 'unsafe'; }
function guardedAfterArguments(table) {
  const allowed = new Set(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  const changed = mutate(arguments), statement = `/* guardedAfterArguments */ SELECT 1 FROM ${table}`;
  return database.query(statement);
}
module.exports = { guardedAfterArguments };
