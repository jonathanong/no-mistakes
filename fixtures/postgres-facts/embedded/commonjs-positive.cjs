const database = require('@example/db');
database.query('SELECT 1');
const alias = database;
alias.query('SELECT 2');
const { query: execute } = database;
execute('SELECT 3');
require('@example/db').query('SELECT 4');
// A configured module namespace itself is not a callable SQL executor.
database('SELECT 5');
alias('SELECT 6');
require('@example/db')('SELECT 7');
