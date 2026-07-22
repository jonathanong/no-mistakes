import * as fs from 'node:fs';
import * as fsPromises from 'node:fs/promises';
import * as url from 'node:url';
import { fileURLToPath } from 'node:url';
import { glob, globSync } from 'glob';
import * as globNamespace from 'glob';
import fastGlob from 'fast-glob';
import { readFile as directRead, readFileSync, readdirSync } from 'fs';
import { readFile as promiseRead } from 'node:fs/promises';

// This fixture intentionally drives the conservative extractor branches that
// prevent a dynamic runtime resource from becoming a speculative graph edge.
function resourceScope(dynamicPath: string, dynamicCwd: string) {
  directRead('direct-import.json');
  readFileSync('direct-sync-import.json');
  readdirSync('direct-directory-import.json');
  promiseRead('promises-import.json');
  fsPromises.readdir('promises-namespace-directory.json');
  fs.promises.readFile('nested-fs-promises.json');
  fs.readFile(dynamicPath);
  glob(dynamicPath);
  glob('templates/**/*.txt', { cwd: dynamicCwd });
  glob('templates/**/*.txt', { cwd: `static-cwd` });
  glob('templates/**/*.txt', { cwd: import.meta.dirname });
  glob('templates/**/*.txt');
  globSync('templates/**/*.txt', { cwd: ('parenthesized-cwd') });
  globNamespace.glob('namespace/**/*.txt');
  fastGlob.sync('fast-glob/**/*.txt');

  require('node:fs').readFile('inline-require.json');
  require('node:fs').promises.readFile('inline-promises-require.json');
  require('glob')('inline-default-glob/**/*.txt');
  require('glob').sync('inline-glob-sync/**/*.txt');

  fs.readFile(new URL('./url-resource.json', import.meta.url));
  fs.readFile(fileURLToPath(new URL('./file-url-resource.json', import.meta.url)));
  fs.readFile(url.fileURLToPath(new URL('./namespace-url-resource.json', import.meta.url)));
  fs.readFile(require('node:url').fileURLToPath(new URL('./inline-url-resource.json', import.meta.url)));

  // A hoisted var is a function-scoped shadow but does not leak outside.
  if (condition) {
    var local = value;
  }
  fs.readFile('after-var-binding.json');

  for (let readFile of []) {
    readFile('shadowed-in-for.json');
  }
  for (const fs of []) {
    fs.readFile('shadowed-in-for-of.json');
  }
  for (const glob in {}) {
    glob('shadowed-in-for-in.json');
  }
  [1].map(() => fs.readFile('anonymous-arrow.json'));
  switch (value) {
    case 1:
      fs.readFile('switch-resource.json');
      break;
  }
}

const read = fs.readFile;
read = customRead;
read('must-not-be-recorded.json');

resourceScope(pathValue, cwdValue);

export default function namedDefault() {
  fs.readFile('named-default.json');
}
