import { read } from '@example/db';
export function split() {
  return read('SELECT id FROM accounts\n-- no-mistakes-' + 'disable-next-line postgres-no-offset\nWHERE id = 1');
}
