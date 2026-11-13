import { readdirSync } from 'node:fs';

function parameter({ excluded, ...rest }) {
  readdirSync(rest.root, 'parameter-member');
  readdirSync(rest.excluded, 'excluded-member');
}
function emptyDefault({ excluded, ...rest } = {}) {
  readdirSync(rest.root, 'empty-default');
}
function populatedDefault({ excluded, ...rest } = { root: 'fixtures' }) {
  readdirSync(rest.root, 'populated-default');
}
function computedParameter({ [key]: excluded, ...rest }) {
  readdirSync(rest.root, 'computed-pattern');
}
function local(options) {
  const { excluded, ...rest } = options;
  readdirSync(rest.root, 'local-parameter');
}
function localObject() {
  const { excluded, ...rest } = { excluded: 'src', root: 'fixtures' };
  readdirSync(rest.root, 'local-object');
}
function localExcluded() {
  const { root, ...rest } = { root: 'src' };
  readdirSync(rest.root, 'local-excluded');
}
function reassigned(options) {
  const { excluded, ...rest } = options;
  options = { root: 'src' };
  readdirSync(rest.root, 'source-reassigned');
}
function escaped(options) {
  const { excluded, ...rest } = options;
  consume(rest);
  readdirSync(rest.root, 'rest-escaped');
}
function mutated(options) {
  const { excluded, ...rest } = options;
  rest.root = 'src';
  readdirSync(rest.root, 'rest-written');
}
function spreadExcluded(options) {
  const { root, ...rest } = options;
  readdirSync({ root: 'fixtures', ...rest }, 'spread-excluded');
}
function spreadPresent(options) {
  const { other, ...rest } = options;
  readdirSync({ root: 'fixtures', ...rest }, 'spread-present');
}
function spreadComputed(options) {
  const { [key]: ignored, ...rest } = options;
  readdirSync({ root: 'fixtures', ...rest }, 'spread-computed');
}
function spreadParameter({ root, ...rest }) {
  readdirSync({ root: 'fixtures', ...rest }, 'spread-parameter');
}
function spreadUnsafe(options) {
  const { root, ...rest } = options;
  consume(rest);
  readdirSync({ root: 'fixtures', ...rest }, 'spread-unsafe');
}
function renamed({ root: removed, ...rest }) {
  readdirSync(rest.root, 'renamed-exclusion');
}
function nested({ options: { ignored, ...rest } }) {
  readdirSync(rest.root, 'nested-pattern');
}
