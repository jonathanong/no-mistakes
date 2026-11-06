import { read } from '@bench/db';
import sql from 'sql-template-strings';
import { selectWithOwner } from './broken.mjs';
const query = selectWithOwner(sql`/* returned */ SELECT `);
read(query.text, query.values);
