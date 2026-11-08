import { read } from '@example/db';
export function direct() { return read('SELECT id FROM accounts OFFSET 20'); }
