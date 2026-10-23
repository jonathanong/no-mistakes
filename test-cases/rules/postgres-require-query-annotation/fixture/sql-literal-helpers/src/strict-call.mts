import { query } from 'pg';
import { literal } from './strict-wrapper.mjs';
const sql = '/* known identifier */ SELECT 1';
query(literal('/* direct literal */ SELECT 1'));
query(literal(sql));
query(literal('/* extra argument */ SELECT 1', 0));
