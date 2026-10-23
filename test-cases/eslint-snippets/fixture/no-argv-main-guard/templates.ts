import { argv } from "node:process";
import { resolve } from "node:path";

if (import.meta.url === `file://${process.argv[1]}`) main();
if (`file://${process.argv?.[1]}` !== import.meta.url) main();
if (import.meta.url == `file://${process.argv.at(1)}`) main();
if (`file://${argv[1]}` != import.meta.url) main();
const invoked = process.argv[1];
const invokedUrl = `file://${invoked}`;
if (invokedUrl === import.meta.url) main();
if (import.meta.url === `file://${(process.argv[1] as string)!}`) main();
if (import.meta.url === `file://${resolve(process.argv[1])}`) main();
if (Boolean(import.meta.url === `file://${process.argv[1]}`)) main();
// Compare decoded template characters, rather than raw escape spelling.
if (import.meta.url === `file\x3a//${process.argv[1]}`) main();
