import {
  readFile as arrowRead,
  readFile as objectRead,
  readFile as arrayRead,
  readFile as activeRead,
  readFile as nestedRead,
  readdir as restRead,
} from "node:fs/promises";

export const arrowCase = () => {
  // `var` is hoisted through nested control flow and shadows the import above.
  arrowRead("ignored-arrow.txt");
  if (globalThis.condition) {
    var arrowRead = () => {};
  }
};

// Every identifier written through a destructuring target invalidates its alias.
({ value: objectRead, ...restRead } = globalThis.objectReplacement);
[arrayRead, { value: nestedRead = () => {} }] = globalThis.arrayReplacement;
objectRead("ignored-object.txt");
restRead("ignored-rest.txt");
arrayRead("ignored-array.txt");
nestedRead("ignored-nested.txt");

export const api = {
  nested: {
    async load() {
      return activeRead("nested.txt");
    },
    deeper: {
      load: async () => activeRead("deep.txt"),
    },
  },
};

// Data-property and static-field initializers execute eagerly when this module
// loads, even though their containing aggregates are private.
const privateCache = {
  schema: activeRead("eager-object.txt"),
  nested: {
    schema: activeRead("eager-nested-object.txt"),
  },
};

class PrivateCache {
  static schema = activeRead("eager-static-field.txt");
  instanceSchema = activeRead("deferred-instance-field.txt");

  load() {
    return activeRead("deferred-class-method.txt");
  }

  static loadLater = () => activeRead("deferred-static-arrow.txt");
  static loadFunction = function () {
    return activeRead("deferred-static-function.txt");
  };
}

void privateCache;
void PrivateCache;
