// no-mistakes-disable-file no-top-level-await -- entrypoint
await initialize();
for await (const item of stream) {}
