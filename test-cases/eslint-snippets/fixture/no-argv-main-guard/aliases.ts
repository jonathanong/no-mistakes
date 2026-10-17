import { fileURLToPath } from 'node:url';
const entry = process.argv[1];
const own = fileURLToPath(import.meta.url);
const entryAgain = entry;
if (entryAgain === own) main();
function later() { if (entry !== own) main(); }
function shadow(entryAgain: string) { if (entryAgain === own) main(); }
const argv = process.argv;
if (argv.at(1) === import.meta.filename) main();
const nodeProcess = process;
if (nodeProcess.argv[1] === __filename) main();
let replaced = process.argv[1];
replaced = userInput;
if (replaced === own) main();
const choice = flag ? entry : other;
if (choice === own) main();
const [destructured] = process.argv;
if (destructured === own) main();
const wrapped = process.argv[1] as string;
if (wrapped === (import.meta.filename as string)) main();
