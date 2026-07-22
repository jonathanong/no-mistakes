import fs from 'node:fs';
import glob from 'glob';

export default () => fs.readFile('default-arrow.json');

const api = {
  load() {
    fs.readFile('object-method.json');
  },
};

class Service {
  load() {
    fs.readFile('class-method.json');
  }
}

function lexicalScopes() {
  {
    // The TDZ local intentionally hides the imported fs before its declaration.
    fs.readFile('hidden-by-block.json');
    const fs = local;
  }
  fs.readFile('after-block.json');
}

try {
  throw new Error('ignored');
} catch (fs) {
  fs.readFile('hidden-by-catch.json');
}

function varShadow() {
  fs.readFile('hidden-by-var.json');
  var fs = local;
}

const read = require('fs').readFile;
const promises = require('fs').promises;
const { readFile: fromPromises } = require('fs').promises;
read('member-alias.json');
promises.readFile('promises-alias.json');
fromPromises('promises-destructure.json');

{
  // Reassigning this inner alias must not invalidate the outer promises binding.
  let promises = local;
  promises = custom;
  promises.readFile('hidden-by-inner-reassignment.json');
}
promises.readFile('outer-after-inner-reassignment.json');

glob('templates/**/*.txt', { cwd: 'first', cwd: 'last' });
glob('templates/**/*.txt', { cwd: 'safe', ...options });
glob('templates/**/*.txt', { [key]: 'cwd' });
