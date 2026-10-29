import { pool } from '@example/db';
const { query: run, read, write, execute } = pool;
run('DROP TABLE query_table');
read('DROP TABLE read_table');
write('TRUNCATE write_table');
execute('DROP TABLE execute_table');
