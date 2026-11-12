import { readdirSync } from 'node:fs';
import { inspect } from '@example/inspect';
function literalKey(options) {
  if (options['root'] === 'fixtures') {}
  readdirSync(options.root, 'literal-key');
}
function siblingRead(options) {
  if (options.other.root) {}
  readdirSync(options.paths.root, 'sibling-read');
}
function rootEscape(options) {
  consume(options);
  readdirSync(options.paths.root, 'root-escape');
}
function reassigned(options) {
  options = { paths: { root: 'fixtures' } };
  readdirSync(options.paths.root, 'reassigned');
}
function unknownRoot() {
  readdirSync(UNBOUND.paths.root, 'unknown-root');
}
function dynamicRead(options, key) {
  consume(options[key]);
  readdirSync(options.paths.root, 'dynamic-read');
}
function mutateRoot(options) {
  options.root.push('child');
  readdirSync(options.root, 'mutate-root');
}

function literalEffects(options) {
  options['root'].push('child');
  inspect(options, 'literal-effects');
}
