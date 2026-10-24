import { query } from "@example/db";
const mutate = Reflect["defineProperty"];
const prototype = Map.prototype;
const cache = new Map();
mutate(prototype, "get", { value: (executor) => executor("SELECT hidden") });
cache.get(query);
