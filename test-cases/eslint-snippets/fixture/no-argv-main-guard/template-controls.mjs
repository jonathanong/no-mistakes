// Only the exact file URL construction retains the argv identity we judge.
if (import.meta.url === `https://${process.argv[1]}`) main();
if (import.meta.url === `file://${process.argv[1]}/suffix`) main();
if (import.meta.url === `file://${process.argv[1]}${""}`) main();
if (import.meta.url === `file://${process.argv[2]}`) main();
if (import.meta.url === `file://${import.meta.filename}`) main();
if (import.meta.url === `file://`) main();
if (import.meta.url === process.argv[1]) main(); // no-mistakes-disable-line no-argv-main-guard -- existing control
if (import.meta.url === tag`file://${process.argv[1]}`) main();
if (import.meta.url === `file://${resolve(process.argv[1])}`) main();
function resolve(value) { return value; }
function local(process) {
  if (import.meta.url === `file://${process.argv[1]}`) main();
}
let invoked = process.argv[1];
invoked = "different";
if (import.meta.url === `file://${invoked}`) main();
if (import.meta.url < `file://${process.argv[1]}`) main();
