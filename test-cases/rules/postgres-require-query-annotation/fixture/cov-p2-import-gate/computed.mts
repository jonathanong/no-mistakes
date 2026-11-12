import { read } from '@bench/db';
const loaded = import(moduleName);
read(`/* ${flag ? 'first' : 'second'} */ SELECT 1`);
read('/* ordinary */ SELECT 2');
read('SELECT 3');
