import proc = require("node:process");

const { argv: args } = require("node:process");
const { ["argv" as const]: wrappedArgv } = proc;

if (args[1] === import.meta.filename) console.log("destructure");
if (proc.argv[1] === import.meta.filename) console.log("equals");
if (proc.argv[1 as const] === (import.meta as ImportMeta).filename) console.log("index");
if (proc.argv.at(1 as const) === import.meta.filename) console.log("at");
if (import.meta.url === new (URL as typeof URL)(`file://${wrappedArgv[1]}`).href) console.log("url");
