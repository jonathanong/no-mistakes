import { read } from '@example/db';
export function unrelated() { return read('SELECT id FROM accounts OFFSET 40'); }
