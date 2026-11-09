const { selected } = await import(new URL('./target.mts', import.meta.url).pathname);
selected();
