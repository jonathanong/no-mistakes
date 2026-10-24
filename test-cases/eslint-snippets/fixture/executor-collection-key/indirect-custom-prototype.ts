import { query } from "@example/db";
declare const custom: { prototype: object };
declare function opaque(value: unknown): void;
const cache = new Map();
opaque(custom.prototype);
cache.get(query);
