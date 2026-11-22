import { createRequire } from "node:module";
const load = createRequire(import.meta.url);
const specifier = process.env.MODULE;
load(specifier);
load.resolve(specifier);
load("./value.mts");
load.resolve("./value.mts");
