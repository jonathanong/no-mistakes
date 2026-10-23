import { query } from '@example/db';
import { run } from './wrap';
// A direct excluded call must not hide SQL forwarded into the included wrapper.
query('SELECT 2');
run('SELECT * FROM unannotated');
