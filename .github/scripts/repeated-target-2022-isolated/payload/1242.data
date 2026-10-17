import sql from './default-executor.mjs';
import { execute as alias } from './default-executor-barrel.mjs';
for (const item of ['one']) { sql(item); }
for (const item of ['one']) { alias`SELECT ${item}`; }
