import { readFileSync } from 'node:fs';

export const api = {
  load() {
    function unused() {
      return readFileSync('resources/exported-object-unused.txt', 'utf8');
    }
    return readFileSync('resources/exported-object.txt', 'utf8');
  },
};

export const eagerApi = {
  schema: readFileSync('resources/exported-named-root.txt', 'utf8'),
};

export const Service = class {
  load() {
    return readFileSync('resources/exported-class-expression.txt', 'utf8');
  }
};

export class NamedService {
  load() {
    return readFileSync('resources/exported-named-class.txt', 'utf8');
  }
}

export default {
  schema: readFileSync('resources/exported-default-root.txt', 'utf8'),
};
