// This lexical import is a factory, not the CommonJS module loader.
import { require } from "../helpers/require-factory";
const loaded = require("node:fs");
loaded.readFileSync("fixtures/a.txt");
require.resolve("node:fs");
// This must retain the imported factory identity, not a synthetic node:fs target.
const { readdirSync: scan } = require("node:fs");
scan("src");
