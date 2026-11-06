import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { page } from './broken.mjs';
page(sql`/* page */ SELECT 1`, statement => read(statement));
