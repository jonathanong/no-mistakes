// no-mistakes-disable-next-line no-top-level-await -- entrypoint
await initialize();
await initialize(); // no-mistakes-disable-line no-mistakes/no-top-level-await -- entrypoint
const text = "no-mistakes-disable-file no-top-level-await";
await initialize();
