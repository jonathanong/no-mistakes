import { connection } from '@example/db';
connection.query('SELECT 1');
const alias = connection;
alias.query('SELECT 2');
