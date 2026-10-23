import * as db from '@example/db';
db.read('SELECT 1 OFFSET 1');
db['read']('SELECT 1 OFFSET 2');
db.query('SELECT 1 OFFSET 3');
