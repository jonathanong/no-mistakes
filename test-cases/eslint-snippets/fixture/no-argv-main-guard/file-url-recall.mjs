// Every spelling keeps one argv identity behind a static file URL prefix.
if (import.meta.url === "file://" + process.argv[1]) main();
if (import.meta.url === `file://${process.argv[1]}`.toString()) main();
if (import.meta.url === String(`file://${process.argv[1]}`)) main();
if (import.meta.url === new URL(`file://${process.argv[1]}`).href) main();
if (import.meta.url === `${"file://"}${process.argv[1]}`) main();
if (import.meta.url === `file://${process.argv[1]}` + "") main();
if (import.meta.url === `file://${process.argv[1] ?? ""}`) main();
if (import.meta.url === `file:///${process.argv[1]}`) main();
if (import.meta.url === `FILE://${process.argv[1]}`) main();
const prefix = "fi" + "le:";
const slashes = `//`;
const invoked = process.argv[1];
const invokedUrl = prefix + (slashes + invoked);
if (invokedUrl === import.meta.url) main();
if (import.meta.url === `${prefix}${slashes}${invoked}`) main();
if (import.meta.url === String(prefix) + invoked.toString()) main(); // Incomplete prefix: deliberately quiet.
if (import.meta.url === new URL("FILE:///" + (invoked ?? "")).href.toString()) main();
if (import.meta.url === ("file://" + invoked).toString()) main();
if (import.meta.url === `${"file://".toString()}${invoked}`) main();
if (import.meta.url === "file://" + String(invoked)) main();
// An empty outer template does not add a second file URL prefix.
if (import.meta.url === `${invokedUrl}`) main();
// Eight-character prefixes remain accepted at the folding boundary.
if (import.meta.url === ("file:" + "///") + process.argv[1]) main();
