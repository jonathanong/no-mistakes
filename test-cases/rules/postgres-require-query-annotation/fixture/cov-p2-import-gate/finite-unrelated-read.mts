import {read} from '@bench/db';
import sql from 'sql-template-strings';
function known(name) { const q = sql``.append(`/* known:${name} */ SELECT 1`); read(q); } // low: true
function caller() { const select = name => known(name); select('first'); }
read(unknownQuery); // low: false
