import { read } from '@bench/db';
import { factory } from './async-factory.mjs';
const result = await factory();
read(result);
