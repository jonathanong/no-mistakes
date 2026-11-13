import { read } from '@bench/db';
import './missing.mjs';
read('/* ordinary */ SELECT 2');
read('SELECT 3');
