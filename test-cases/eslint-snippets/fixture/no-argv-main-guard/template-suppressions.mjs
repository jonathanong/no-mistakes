// no-mistakes-disable-next-line no-argv-main-guard -- deliberate legacy guard
if (import.meta.url === `file://${process.argv[1]}`) main();
if (import.meta.url === `file://${process.argv[1]}`) main(); // no-mistakes-disable-line no-argv-main-guard -- deliberate legacy guard
// Directive-looking text is data and cannot suppress a finding.
const text = "no-mistakes-disable-file no-argv-main-guard";
if (import.meta.url === `file://${process.argv[1]}`) main();
