import { createRequire } from "node:module";
module.require("walker");
const load = createRequire(import.meta.url);
load("walker");
