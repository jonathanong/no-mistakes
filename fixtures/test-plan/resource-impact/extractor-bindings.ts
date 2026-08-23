import fs, { readFileSync as read, promises } from 'node:fs';
import fg from 'fast-glob';
import { glob, globSync } from 'glob';
import { fileURLToPath as file } from 'node:url';

fs.readFile('schema.sql');
read(`sync.sql`);
promises.readdir('migrations');
fg('templates/**/*.hbs', { cwd: 'src' });
glob('**/*.json');
globSync('**/*.md');
fs.readFile(file(new URL('./local.json', import.meta.url)));
