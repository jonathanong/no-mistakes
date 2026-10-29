import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const where = require.resolve("node:path");
