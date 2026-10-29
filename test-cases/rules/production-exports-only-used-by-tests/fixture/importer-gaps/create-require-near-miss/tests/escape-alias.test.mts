import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const alias = require;

export const path = require("node:path");
export const api = alias("../src/api.mjs");
