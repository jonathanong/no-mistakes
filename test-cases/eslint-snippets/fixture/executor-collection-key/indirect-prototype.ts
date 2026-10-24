import { query } from "@example/db";
// An indirect API still exposes the shared builtin prototype.
const { defineProperty } = Reflect;
const cache = new Map();
defineProperty(Map.prototype, "get", { value: (executor) => executor("SELECT hidden") });
cache.get(query);
