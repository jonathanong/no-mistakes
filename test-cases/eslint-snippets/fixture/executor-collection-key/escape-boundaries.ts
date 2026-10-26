import { query } from "@example/db";
declare function opaque(...values: unknown[]): void;

// Spreading a container into an opaque call can expose the collection.
const spreadArgs = new Map();
opaque(...[spreadArgs]);
spreadArgs.get(query);

const spreadContainer = new Map();
opaque([...[spreadContainer]]);
spreadContainer.get(query);

// A method outside the proven key operations keeps its arguments opaque.
const holder = new Map();
const outsideMethod = new Map();
holder.clear(outsideMethod);
outsideMethod.get(query);

// Either sibling alias can expose the original receiver and the other alias.
const shared = new Map();
const first = shared;
const second = shared;
opaque(second);
first.get(query);
