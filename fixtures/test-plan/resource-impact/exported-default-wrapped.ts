import { readFileSync } from 'node:fs';

export default Object.freeze({
  config: readFileSync('resources/exported-default-wrapped.txt', 'utf8'),
});
