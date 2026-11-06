import { read } from '@bench/db';
import { statement } from './broken.mjs';
read(statement());
