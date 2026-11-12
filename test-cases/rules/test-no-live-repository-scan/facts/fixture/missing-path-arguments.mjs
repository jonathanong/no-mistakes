import { readdirSync } from 'node:fs';

readdirSync(...unknownArguments);
readdirSync(...[]);
readdirSync();
readdirSync(unknownRoot);
readdirSync(...['fixtures/input']);
readdirSync(...['src']);

function forwarded(...roots) {
  return readdirSync(...roots);
}
