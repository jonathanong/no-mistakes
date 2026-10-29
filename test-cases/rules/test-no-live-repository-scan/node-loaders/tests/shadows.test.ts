import { createRequire } from "../helpers/custom";
createRequire("url")("node:fs");
const fake = { createRequire: (_url: string) => (_name: string) => ({}) };
fake.createRequire("url")("node:fs");
function custom(require: (name: string) => { createRequire: typeof createRequire }) {
  const { createRequire: make } = require("node:module");
  make("url")("node:fs");
}
custom(() => ({ createRequire }));
const module = { require: (_name: string) => ({}) };
module.require("node:fs");
