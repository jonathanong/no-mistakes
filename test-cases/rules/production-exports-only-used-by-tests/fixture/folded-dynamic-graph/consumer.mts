const entry = './target.mts';
const { selected } = await import(entry);
selected();
