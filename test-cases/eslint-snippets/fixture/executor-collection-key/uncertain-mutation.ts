import { query, type QueryExecutor } from "@example/db";

declare const source: object;
const map = new Map<QueryExecutor, unknown>();
Object.assign(map, source);
map.get(query);

const spreadMap = new Map<QueryExecutor, unknown>();
Object.assign(spreadMap, { ...source });
spreadMap.get(query);

const computedMap = new Map<QueryExecutor, unknown>();
Object.assign(computedMap, { ["get"]: () => undefined });
computedMap.get(query);

declare const dynamicProperty: string;
const dynamicMap = new Map<QueryExecutor, unknown>();
Object.defineProperty(dynamicMap, dynamicProperty, { value: undefined });
dynamicMap.get(query);

const computedPropertyMap = new Map<QueryExecutor, unknown>();
Object.assign(computedPropertyMap, { [dynamicProperty]: () => undefined });
computedPropertyMap.get(query);
