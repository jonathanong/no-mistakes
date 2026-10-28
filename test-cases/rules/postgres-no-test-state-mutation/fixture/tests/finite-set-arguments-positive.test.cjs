const database = require('@example/db');

function guardedWithoutAlias(table) {
  const allowed = new Set(['safe_one', 'safe_two']);
  if (!allowed.has(table)) throw new Error('invalid table');
  const statement = `/* guardedWithoutAlias */ SELECT 1 FROM ${table}`;
  return database.query(statement);
}
module.exports = { guardedWithoutAlias };
