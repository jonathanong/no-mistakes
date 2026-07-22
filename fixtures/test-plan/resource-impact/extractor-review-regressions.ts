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
