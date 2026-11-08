import { query } from 'db';
function expression() { return 'fn(id)'; }
query('SELECT 1 FROM accounts WHERE ' + expression() + ' = $1');
