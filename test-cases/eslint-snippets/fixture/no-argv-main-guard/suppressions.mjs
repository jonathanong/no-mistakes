// no-mistakes-disable-next-line no-argv-main-guard -- test an intentionally old entrypoint
if (process.argv[1] === import.meta.filename) main();
if (process.argv[1] === __filename) main(); // no-mistakes-disable-line no-mistakes/no-argv-main-guard -- fixture
const text = 'no-mistakes-disable-file no-argv-main-guard';
if (process.argv[1] === import.meta.url) main();
