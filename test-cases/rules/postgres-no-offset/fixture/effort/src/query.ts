import { query } from '@example/db';
query('SELECT 1 OFFSET 2');
query(`SELECT 1
OFFSET 0`);
const saved = `SELECT 1 OFFSET 4`;
query(saved);
query(saved);
function build() { return 'SELECT 1 OFFSET 8'; }
query(build());
declare const tail: string;
query(`SELECT 1 ${tail}`);
query('SELECT 1\nOFFSET 9');
query('SELECT 1');
