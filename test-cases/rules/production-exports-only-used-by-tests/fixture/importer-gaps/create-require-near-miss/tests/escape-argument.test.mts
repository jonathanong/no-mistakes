import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const use = (load: unknown) => load;

export const path = require("node:path");
export const loader = use(require);
