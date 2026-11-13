import { read } from '@bench/db';
import { enabled } from './missing.mjs';
read(`/* ${enabled ? 'first */ SELECT 1 /* tail' : 'second */ SELECT 2 /* tail'} */`);
read('/* ordinary */ SELECT 2');
read('SELECT 3');
