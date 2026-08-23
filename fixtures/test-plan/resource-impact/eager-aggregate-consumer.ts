import { readFileSync } from 'node:fs';

const cache = {
  schema: readFileSync('resources/eager-object.txt', 'utf8'),
};

class Cache {
  static schema = readFileSync('resources/eager-static-field.txt', 'utf8');

  load() {
    return readFileSync('resources/eager-deferred-method.txt', 'utf8');
  }
}

export const eagerSchemas = [cache.schema, Cache.schema];
