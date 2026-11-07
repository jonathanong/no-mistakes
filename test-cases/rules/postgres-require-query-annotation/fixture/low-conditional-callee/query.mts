import { write, read } from '@app/db';
import { read as otherRead } from '@other/db';

export function run(condition: boolean, dynamic: string) {
  (condition ? write : read)('/* user/load */ SELECT 1');
  (condition ? write : read)('SELECT 1');
  (condition ? write : read)(dynamic);
  const execute = condition ? write : read;
  execute('/* user/load */ SELECT 2');
  execute('SELECT 2');
  (condition ? write : otherRead)('SELECT 3');
  function shadowed(read: (sql: string) => unknown) {
    (condition ? write : read)('SELECT 4');
  }
  return shadowed;
}
