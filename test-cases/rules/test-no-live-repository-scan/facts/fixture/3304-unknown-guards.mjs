import { readdirSync } from 'node:fs';
function globalCondition() {
  if (MODE === 'scan') readdirSync('fixtures', 'global-condition');
}
function globalArray() {
  const [mode, directory] = GLOBAL_ARGS;
  if (mode === 'scan') readdirSync(directory, 'global-array');
}
function globalAlias() {
  const mode = GLOBAL_MODE;
  if (mode === 'scan') readdirSync('fixtures', 'global-alias');
}
function globalRest() {
  const [mode, ...rest] = GLOBAL_REST;
  if (rest.length === 1) readdirSync(rest[0], 'global-rest');
}
function parameter(args) {
  const [mode, directory] = args;
  if (mode === 'scan') readdirSync(directory, 'parameter');
}
