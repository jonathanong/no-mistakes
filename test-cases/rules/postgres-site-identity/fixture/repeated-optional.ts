import { query } from 'db';
function transform(column: string) { return 'fn(' + column + ')'; }
export function run(flag: boolean) {
  query('SELECT 1 FROM accounts WHERE ' + (flag ? transform('id') + ' = $1 AND ' : '') + transform('token') + ' = $2');
}
