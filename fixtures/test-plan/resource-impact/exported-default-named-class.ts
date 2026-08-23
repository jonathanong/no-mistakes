import { readFileSync } from 'node:fs';

export default class Service {
  load() {
    return readFileSync('resources/exported-default-named-class.txt', 'utf8');
  }
}
