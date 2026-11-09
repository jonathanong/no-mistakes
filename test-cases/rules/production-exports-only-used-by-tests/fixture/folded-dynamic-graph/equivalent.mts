const { selected: first } = await import(new URL('./target.mts', import.meta.url)); const { selected: second } = await import(new URL('./target.mts', import.meta.url)); first(); second();
