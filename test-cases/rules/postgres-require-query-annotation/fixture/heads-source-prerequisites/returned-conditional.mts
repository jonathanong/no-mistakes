import { read } from '@bench/db';
import sql from 'sql-template-strings';
import { suffix } from './literal-helper.mjs';
import { selectWithOwner } from './broken.mjs';
const query = choose
  ? suffix(sql`/* healthyArm */ SELECT `)
  : selectWithOwner(sql`/* brokenArm */ SELECT `);
read(query.text, query.values);
