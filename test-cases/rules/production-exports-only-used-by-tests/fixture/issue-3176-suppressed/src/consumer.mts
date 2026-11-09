const entry = process.env.RUNTIME_ENTRY;
await import(entry); // no-mistakes-disable-line production-exports-only-used-by-tests
