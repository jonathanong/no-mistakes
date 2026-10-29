import { createRequire } from "node:module";

const load = createRequire(import.meta.url);

export const api = load("./api");
