import { query } from '@example/db';

declare const condition: boolean;
query(condition ? 'CREATE TABLE variant_test (id integer)' : 'SELECT 1');
