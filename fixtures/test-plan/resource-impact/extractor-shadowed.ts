import { readFile } from 'fs';

function local(readFile: (path: string) => void) {
  readFile('not-a-resource');
}

readFile = custom;
readFile('also-not-a-resource');
