import {read} from '@bench/db';
import sql from 'sql-template-strings';
function unused(name) { const q = sql``.append(`/* unused:${name} */ SELECT 1`); void q; }
read('/* ordinary */ SELECT 1'); // low: true
