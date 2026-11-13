import {read} from '@bench/db';
for (const name of ['first', 'second']) { other(`/* ${name} */ SELECT 1`); }
read('/* ordinary */ SELECT 1'); // low: true
