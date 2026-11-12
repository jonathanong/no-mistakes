import { read } from '@bench/db';
import { enabled } from './missing.mjs';
read(`/* ${flag ? 'first' : 'second'} */ SELECT 1`);
read('/* ordinary */ SELECT 2');
read('SELECT 3');
