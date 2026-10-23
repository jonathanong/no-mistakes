// Keep altered paths and same-spelled local APIs unknown.
if (import.meta.url === "https://" + process.argv[1]) main();
if (import.meta.url === "file:/" + process.argv[1]) main();
if (import.meta.url === "file:////" + process.argv[1]) main();
if (import.meta.url === "file://" + process.argv[1] + "/suffix") main();
if (import.meta.url === "file://" + process.argv[2]) main();
if (import.meta.url === "file://" + import.meta.filename) main();
if (import.meta.url === `file://${process.argv[1] ?? "fallback"}`) main();
if (import.meta.url === `${process.argv[1]}file://`) main();
if (import.meta.url === `file://${process.argv[1]}${process.argv[1]}`) main();
if (import.meta.url === `file://${process.argv[1]}${""}`) main();
if (import.meta.url === process.argv[1] + process.argv[1]) main();
if (import.meta.url === `file://${process.argv[1]}`.toString("changed")) main();
if (import.meta.url === new URL(process.argv[1]).href) main();
if (import.meta.url === new URL(`file://${process.argv[1]}`, import.meta.url).href) main();
if (import.meta.url === String(...[`file://${process.argv[1]}`])) main();
if (import.meta.url === process.argv.toString()) main();
if (import.meta.url === `file://${process.argv[1]}` - "") main();
if (import.meta.url === `file://${process.argv[1] || ""}`) main();
if (import.meta.url === String(`file://${process.argv[1]}`, "ignored")) main();
function shadowed(String, URL, process) {
  if (import.meta.url === String(`file://${process.argv[1]}`)) main();
  if (import.meta.url === new URL(`file://${process.argv[1]}`).href) main();
}
function shadowedWrappers(String, URL) {
  if (import.meta.url === String(`file://${process.argv[1]}`)) main();
  if (import.meta.url === new URL(`file://${process.argv[1]}`).href) main();
}
const other = { toString() { return process.argv[1]; } };
if (import.meta.url === "file://" + other.toString()) main();
if (import.meta.url === new Other(`file://${process.argv[1]}`).href) main();
const changed = "file://" + process.argv[1];
// Const bindings are stable, but arbitrary method names do not preserve identity.
if (import.meta.url === changed.replace("file:", "https:")) main();
let prefix = "file://";
prefix = "https://";
if (import.meta.url === prefix + process.argv[1]) main();
// A URL object itself is not the URL's href string.
if (import.meta.url === new URL(`file://${process.argv[1]}`)) main();
if (import.meta.url === new URL(new URL(`file://${process.argv[1]}`)).href) main();
const prefixed = "file://" + process.argv[1];
// Preserve the inner prefix so this cannot become a direct argv identity.
if (import.meta.url === `file://${prefixed}`) main();
