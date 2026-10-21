import { argv } from "node:process";
import url from "node:url";

if (argv[1] === import.meta.filename) main();
if (process.argv[`1`] === import.meta.url) main();
if (process.argv[1] === url.fileURLToPath(import.meta.url)) main();
const { argv: args } = process;
if (args[1] === import.meta.filename) main();
const { ["argv"]: bracketArgs, ...ignored } = process;
if (bracketArgs[1] === import.meta.filename) main();
void ignored;
const { [`argv`]: templateArgs } = process;
if (templateArgs[1] === import.meta.filename) main();
if (process.argv[1] === url.fileURLToPath.call(null, import.meta.url)) main();
