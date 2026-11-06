import { read } from '@bench/db';
import { remote } from './broken.mjs';
const result = await remote();
read(result);
