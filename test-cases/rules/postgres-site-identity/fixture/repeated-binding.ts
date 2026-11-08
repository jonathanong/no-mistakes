import { query } from 'db';
function expression() { return 'fn(id)'; }
const part = expression();
query('SELECT 1 FROM accounts WHERE ' + part + ' = $1 AND ' + part + ' = $2');
