import { readFileSync } from 'node:fs';

export default class {
  load() {
    return readFileSync('resources/exported-default-anonymous-class.txt', 'utf8');
  }
}
