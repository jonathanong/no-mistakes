import { createRequire } from "node:module";

const load = createRequire(import.meta.url);

export const path = load("node:path");
