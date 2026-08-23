const { readFile: read, promises: { readdir } } = require('fs');
const glob = require('tinyglobby');

read(name);
readdir('untracked-directory');
glob('**/*.ts', { cwd: process.cwd() });
