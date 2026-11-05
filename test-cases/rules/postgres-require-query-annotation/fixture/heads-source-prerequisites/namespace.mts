import { read } from '@bench/db';
import * as helpers from './broken.mjs';
const { statement } = helpers;
read(statement());
