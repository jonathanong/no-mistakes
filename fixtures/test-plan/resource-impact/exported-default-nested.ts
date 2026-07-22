import { readFileSync } from 'node:fs';

export default {
  load() {
    const callbacks = [
      () => readFileSync('resources/exported-default-nested-unused.txt', 'utf8'),
    ];
    return callbacks.length;
  },
};
