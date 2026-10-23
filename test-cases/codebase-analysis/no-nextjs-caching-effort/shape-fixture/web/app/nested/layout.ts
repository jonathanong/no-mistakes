// Only the exported binding names select cache settings, even inside patterns.
export const [{ revalidate }] = settings;
export const { nested: { dynamic } } = settings;
export const { fetchCache = 'auto' } = settings;
export const { unrelated: value } = settings;
