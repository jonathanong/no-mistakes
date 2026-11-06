import { read } from '@bench/db';
import { statement } from './broken.mjs';
read('/* independent */ SELECT 1');
