import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const path = require("node:path");
export const api = require("../src/api.mjs");
