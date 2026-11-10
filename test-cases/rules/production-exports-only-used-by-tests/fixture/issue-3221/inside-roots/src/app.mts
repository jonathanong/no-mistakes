const entry = process.env.ENTRY;
await import(entry);

export const started = true;
