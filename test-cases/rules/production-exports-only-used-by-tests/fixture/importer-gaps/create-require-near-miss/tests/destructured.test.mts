import { createRequire } from "node:module";

const { require } = { require: createRequire(import.meta.url) };

export const path = require("node:path");
