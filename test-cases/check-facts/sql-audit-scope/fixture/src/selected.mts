import { read } from '@example/db';
export function selected() { return read('SELECT id FROM accounts OFFSET 20'); }
