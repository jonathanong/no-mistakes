import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const name = process.env.NAME as string;

export const dynamic = require(name);
