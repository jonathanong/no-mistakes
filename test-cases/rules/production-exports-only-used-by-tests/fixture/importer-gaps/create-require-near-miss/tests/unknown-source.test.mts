// createRequire from a local module is not the Node factory.
import { createRequire } from "./factory.mjs";

const require = createRequire(import.meta.url);

export const path = require("node:path");
