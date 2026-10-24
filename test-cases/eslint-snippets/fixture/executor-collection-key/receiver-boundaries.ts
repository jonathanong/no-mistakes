import { query, type QueryExecutor } from "@example/db";

declare function opaque(value: unknown): void;
const boxed = new Map<QueryExecutor, unknown>();
const box = { boxed };
opaque(box);
boxed.get(query);

const nested = new Map<QueryExecutor, unknown>();
const array = [nested];
opaque(array);
nested.get(query);

const erased = new Map<QueryExecutor, unknown>();
const leak: any = erased;
opaque(leak);
erased.get(query);

const captured = new Map<QueryExecutor, unknown>();
opaque(() => captured);
captured.get(query);

const returned = new Map<QueryExecutor, unknown>();
function returnMap() { return returned; }
returned.get(query);

const arrow = new Map<QueryExecutor, unknown>();
const returnArrow = () => arrow;
arrow.get(query);

// A typed private alias keeps the same proven allocation.
const direct = new Map<QueryExecutor, unknown>();
const directAlias = direct;
directAlias.get(query);

// A type query and a typeof value read do not expose the collection.
const typeOnly = new Map<QueryExecutor, unknown>();
type MapType = typeof typeOnly;
typeOnly.get(query);
const typeofOnly = new Map<QueryExecutor, unknown>();
opaque(typeof typeofOnly);
typeofOnly.get(query);

const memberRead = new Map<QueryExecutor, unknown>();
memberRead.get(query);
const sizeRead = new Map<QueryExecutor, unknown>();
const observedSize = sizeRead.size;
sizeRead.get(query);

// Even a local container removes the proof that this binding stays private.
const storedOnly = new Map<QueryExecutor, unknown>();
const localBox = { storedOnly };
storedOnly.get(query);
const arrayOnly = new Map<QueryExecutor, unknown>();
const localArray = [arrayOnly];
arrayOnly.get(query);
