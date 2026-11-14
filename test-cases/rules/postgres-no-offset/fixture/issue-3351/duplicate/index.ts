import { query } from 'pg';
const duplicate = function (sql, values, sql) { return query(sql, values); };
duplicate('SELECT 1', [], 'SELECT 2');
