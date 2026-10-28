// Runtime imports are hoisted even when their declaration appears below calls.
const loaded = require("node:fs");
require.resolve("node:fs");
import { require } from "../helpers/require-factory";
loaded.readFileSync("fixtures/a.txt");
