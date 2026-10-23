import { query } from 'pg';
import { staticNamed } from './strict-wrapper.mjs';
const sql = '/* known identifier */ SELECT 1';
query(staticNamed('/* direct staticNamed */ SELECT 1'));
query(staticNamed(sql));
query(staticNamed('/* extra argument */ SELECT 1', 0));
