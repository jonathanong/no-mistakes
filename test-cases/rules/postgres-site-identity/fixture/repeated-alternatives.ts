import { query } from 'db';
function transform(column: string) { return 'fn(' + column + ')'; }
export function run(flag: boolean) {
  query('SELECT 1 FROM accounts WHERE ' + (flag ? transform('id') : transform('token')) + ' = $1');
}
