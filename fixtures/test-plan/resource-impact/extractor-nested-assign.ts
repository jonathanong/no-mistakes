import fs from 'node:fs';

let readFile = fs.readFile;
({ nested: [{ readFile = fs.readFile, ...rest } = {}] } = { nested: [{}] });
readFile('after-nested.json');
[{ extra: [also = fs.readFileSync] } = {}] = [{}];
also('after-rest.json');
void rest;

for (x of []) {}
for (y in {}) {}
for (i = 0; i < 1; i++) {}

