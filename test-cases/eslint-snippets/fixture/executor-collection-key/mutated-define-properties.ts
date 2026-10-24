import { query, type QueryExecutor } from "@example/db";

declare const replacement: unknown;
const values = new Map<QueryExecutor, unknown>();
Object.defineProperties(values, { get: { value: replacement } });
values.get(query);

// Unknown descriptors can replace a collection method without naming it here.
declare const descriptors: PropertyDescriptorMap;
const opaque = new Map<QueryExecutor, unknown>();
Object.defineProperties(opaque, descriptors);
opaque.get(query);

declare const dynamicKey: string;
const computed = new Map<QueryExecutor, unknown>();
Object.defineProperties(computed, { [dynamicKey]: { value: replacement } });
computed.get(query);

const spread = new Map<QueryExecutor, unknown>();
Object.defineProperties(spread, { ...descriptors });
spread.get(query);
