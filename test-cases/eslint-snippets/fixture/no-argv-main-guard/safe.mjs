import { fileURLToPath } from 'node:url';
const entry = process.argv[1];
console.log(entry);
if (process.argv[2] === import.meta.filename) main();
if (process.argv.slice(2) === import.meta.url) main();
if (process.argv[1] === '/some/other/file') main();
if (process.argv[1] === userFile) main();
if (process.argv[1] > import.meta.filename) main();
if (process.argv.at(2) === __filename) main();
if (process.argv.at() === __filename) main();
if (process.argv.at(...values) === __filename) main();
if (process.argv[index] === import.meta.filename) main();
if (Object(process.argv[1]) === import.meta.filename) main();
function shadow(process) { if (process.argv[1] === import.meta.filename) main(); }
function fileShadow(__filename) { if (process.argv[1] === __filename) main(); }
function wrapperShadow(fileURLToPath) { if (process.argv[1] === fileURLToPath(import.meta.url)) main(); }
function boolShadow(Boolean) { if (Boolean(process.argv[1]) === import.meta.filename) main(); }
if (fileURLToPath(other, import.meta.url) === process.argv[1]) main();
if (fileURLToPath(...values) === process.argv[1]) main();
if (path.resolve(process.argv[1]) === import.meta.filename) main();
if (import.meta.main) main();
export function injected(isMain = import.meta.main) { if (isMain) main(); }
const [own] = [import.meta.filename];
if (process.argv[1] === own) main();
